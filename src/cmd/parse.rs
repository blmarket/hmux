use crate::cmd::CmdListRef;
use crate::cmd::cmd_get_alias;
use crate::cmd::cmd_parse;
use crate::cmd::{cmd_find_from_client, cmd_find_valid_state};
use crate::cmd::{CmdqItemWeak, CmdqStateRef, cmdq_append};
pub use crate::consts::{
    CMD_PARSE_ERROR, CMD_PARSE_PARSEONLY, CMD_PARSE_SUCCESS, CMD_PARSE_VERBOSE, ENVIRON_HIDDEN,
    FORMAT_NOJOBS, FORMAT_NONE, UINT_MAX,
};
use crate::environ::{EnvironmentStore, with_global_environment, with_global_environment_mut};
use crate::ffi::{getuid, wctomb};
use crate::fmt_args;
use crate::fmt_engine::{FmtArg, format_alloc};
use crate::format::format_true;
use crate::format::{format_create, format_defaults, format_expand};
use crate::log::{fatalx, log_debug};
#[cfg(test)]
use crate::types::cmd_find_state;
pub use crate::types::{
    ArgsValue, ClientRef, cmd_parse_input, cmd_parse_result, cmd_parse_status, size_t, u_int,
    wchar_t,
};
use crate::xmalloc::xasprintf;
use crate::{CommandParser, RustCommandParser};
use crate::{UserAccount, UserAccountRecord};
use ::core::ffi::CStr;

/// The commands of one parse, in the order they were written. Each command
/// belongs to the list.
pub type cmd_parse_commands = Vec<Box<cmd_parse_command>>;

#[repr(C)]
pub struct cmd_parse_command {
    pub line: u_int,
    pub arguments: cmd_parse_arguments,
}

/// The arguments of one command, in the order they were written, and owned by
/// the command the same way.
pub type cmd_parse_arguments = Vec<Box<cmd_parse_argument>>;

#[repr(C)]
pub struct cmd_parse_argument {
    pub type_0: cmd_parse_argument_type,
    pub string: Option<std::ffi::CString>,
    pub commands: Option<Box<cmd_parse_commands>>,
    pub(crate) cmdlist: Option<CmdListRef>,
}
impl cmd_parse_argument {
    /// The list this argument carries, as the borrowed view the walks take,
    /// or null for an argument that carries none.
    fn commands_ptr(&mut self) -> Option<&mut cmd_parse_commands> {
        self.commands.as_deref_mut()
    }
}
pub type cmd_parse_argument_type = core::ffi::c_uint;
pub const CMD_PARSE_PARSED_COMMANDS: cmd_parse_argument_type = 2;
pub const CMD_PARSE_COMMANDS: cmd_parse_argument_type = 1;
pub const CMD_PARSE_STRING: cmd_parse_argument_type = 0;
/// The state of the parser. The default is a parser reading nothing: no
/// input, no scope stack and no error.
#[derive(Default)]
#[repr(C)]
pub struct cmd_parse_state<'a> {
    /// The bytes of a file being parsed, read as unsigned characters the way
    /// `getc` did. A caller's buffer is borrowed for the duration of its
    /// parser turn.
    pub f: Option<Vec<u8>>,
    pub buf: Option<&'a [u8]>,
    pub len: size_t,
    pub off: size_t,
    pub condition: core::ffi::c_int,
    pub eol: core::ffi::c_int,
    pub eof: core::ffi::c_int,
    pub input: Option<&'a mut cmd_parse_input>,
    pub escapes: u_int,
    pub error: Option<std::ffi::CString>,
}
pub type cmd_parse_token_state = core::ffi::c_uint;
pub const SINGLE_QUOTES: cmd_parse_token_state = 3;
pub const DOUBLE_QUOTES: cmd_parse_token_state = 2;
pub const NONE: cmd_parse_token_state = 1;
pub const START: cmd_parse_token_state = 0;

pub const EOF: core::ffi::c_int = -(1 as core::ffi::c_int);

pub const CMD_PARSE_NOALIAS: core::ffi::c_int = 0x4 as core::ffi::c_int;

pub const CMD_PARSE_ONEGROUP: core::ffi::c_int = 0x10 as core::ffi::c_int;

pub const CMD_PARSE_MAX_ENVIRON_LEN: core::ffi::c_int = 16384 as core::ffi::c_int;
impl cmd_parse_state<'_> {
    fn input(&self) -> &cmd_parse_input {
        self.input.as_deref().expect("active parser input")
    }

    fn advance_line(&mut self) {
        let input = self.input.as_deref_mut().expect("active parser input");
        input.line = input.line.wrapping_add(1);
    }
}

type SharedLexer<'a> = std::rc::Rc<std::cell::RefCell<cmd_parse_state<'a>>>;

fn cmd_parse_get_error(file: Option<&CStr>, line: u_int, error: &CStr) -> std::ffi::CString {
    match file {
        None => error.to_owned(),
        Some(file) => xasprintf(c"%s:%u: %s", fmt_args![file, line, error]),
    }
}
unsafe fn cmd_parse_print_commands(pi: &cmd_parse_input, cmdlist: &CmdListRef) {
    unsafe {
        let Some(item) = pi.item.as_ref().and_then(CmdqItemWeak::upgrade) else {
            return;
        };
        if !pi.flags & CMD_PARSE_VERBOSE != 0 {
            return;
        }
        let s = cmdlist.print(0 as core::ffi::c_int);
        if pi.file.is_some() {
            (item.read()).print(c"%s:%u: %s", fmt_args![pi.file(), pi.line, s.as_c_str()]);
        } else {
            (item.read()).print(c"%u: %s", fmt_args![pi.line, s.as_c_str()]);
        }
    }
}
/// The commands of `cmds`, in their stored order, as scoped mutable borrows.
fn command_list(cmds: &mut cmd_parse_commands) -> impl Iterator<Item = &mut cmd_parse_command> {
    cmds.iter_mut().map(Box::as_mut)
}

/// The arguments of one command, in their stored order, as scoped mutable
/// borrows.
fn argument_list(args: &mut cmd_parse_arguments) -> impl Iterator<Item = &mut cmd_parse_argument> {
    args.iter_mut().map(Box::as_mut)
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
pub struct TokenText(std::ffi::CString);
impl TokenText {
    fn from_cstring(text: std::ffi::CString) -> TokenText {
        TokenText(text)
    }
    pub fn as_c_str(&self) -> &CStr {
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
    Word(TokenText),
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
pub struct ParseState<'a> {
    lexer: SharedLexer<'a>,
    scope: Option<bool>,
    stack: Vec<bool>,
}

impl<'a> ParseState<'a> {
    pub fn new() -> Self {
        Self {
            lexer: Default::default(),
            scope: None,
            stack: Vec::new(),
        }
    }

    pub fn line(&self) -> u_int {
        self.lexer.borrow().input().line
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
            let (c, mut fs, item) = {
                let lexer = self.lexer.borrow();
                let pi = lexer.input();
                (
                    pi.client(),
                    pi.fs.clone(),
                    pi.item.as_ref().and_then(CmdqItemWeak::upgrade),
                )
            };
            if cmd_find_valid_state(&fs) == 0 {
                cmd_find_from_client(&mut fs, c.as_ref(), 0 as core::ffi::c_int);
            }
            let session = fs.session();
            let link = fs.winlink_ref();
            let pane = fs.pane_ref();
            let mut ft = match item.as_ref() {
                Some(item) => item
                    .with_item(|item| format_create(None, Some(item), FORMAT_NONE, FORMAT_NOJOBS)),
                None => format_create(None, None, FORMAT_NONE, FORMAT_NOJOBS),
            };
            format_defaults(
                &mut ft,
                c.as_ref().map(|reference| reference.as_client()),
                session.as_ref().map(|reference| reference.as_session()),
                link.as_ref().and_then(|link| link.get()),
                pane.as_ref().and_then(|pane| pane.get()),
            );
            let expanded = format_expand(&mut ft, token.as_c_str());
            TokenText::from_cstring(expanded)
        }
    }

    /// `assignment : EQUALS` and `hidden_assignment : HIDDEN EQUALS`.
    pub fn put_environ(&mut self, token: TokenText, hidden: bool) -> Result<(), LexError> {
        let flags = self.lexer.borrow().input().flags;
        let flag = match self.scope {
            None => true,
            Some(scope) => scope && self.stack.iter().all(|scope| *scope),
        };
        if token.as_c_str().to_bytes().len() > CMD_PARSE_MAX_ENVIRON_LEN as size_t {
            yyerror(
                &mut self.lexer.borrow_mut(),
                c"environment variable is too long",
                fmt_args![],
            );
            return Err(LexError);
        }
        if !flags & CMD_PARSE_PARSEONLY != 0 && flag {
            with_global_environment_mut(|env| {
                env.put(token.as_c_str(), if hidden { ENVIRON_HIDDEN } else { 0 });
            });
        }
        Ok(())
    }

    /// `if_open : IF expanded`.
    pub fn push_scope(&mut self, expanded: &TokenText) -> bool {
        let flag = format_true(Some(expanded.as_c_str())) != 0;
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
        let flag = format_true(Some(expanded.as_c_str())) != 0;
        self.scope = Some(flag);
        flag
    }

    /// `if_close : ENDIF`.
    pub fn pop_scope(&mut self) {
        self.scope = self.stack.pop();
    }
}

impl Default for ParseState<'_> {
    fn default() -> Self {
        ParseState::new()
    }
}

fn cmd_parse_build_arguments(args: &mut cmd_parse_arguments, arguments: Vec<ParseArgument>) {
    args.clear();
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
        args.push(arg);
    }
}

/// Moves an owned parse tree into the lists the command builder reads.
fn cmd_parse_build_list(commands: Vec<ParseCommand>) -> Box<cmd_parse_commands> {
    let mut cmds = Box::new(cmd_parse_commands::new());
    for command in commands {
        let mut cmd = cmd_parse_new_command(command.line);
        cmd_parse_build_arguments(&mut cmd.arguments, command.arguments);
        cmds.push(cmd);
    }
    cmds
}

fn cmd_parse_run_parser(
    lexer: SharedLexer<'_>,
    cause: &mut Option<std::ffi::CString>,
) -> Option<Box<cmd_parse_commands>> {
    let mut ps = ParseState {
        lexer: lexer.clone(),
        scope: None,
        stack: Vec::new(),
    };
    let result = parse_grammar::LinesParser::new().parse(&mut ps, TokenStream(lexer.clone()));
    match result {
        Ok(commands) => Some(cmd_parse_build_list(commands)),
        Err(_) => {
            let mut lexer = lexer.borrow_mut();
            yyerror(&mut lexer, c"syntax error", fmt_args![]);
            *cause = lexer.error.take();
            None
        }
    }
}

/// Pulls one token at a time out of `yylex`, the way `yyparse` did.
struct TokenStream<'a>(SharedLexer<'a>);

impl Iterator for TokenStream<'_> {
    type Item = Result<(usize, Token, usize), LexError>;

    fn next(&mut self) -> Option<Result<(usize, Token, usize), LexError>> {
        match yylex_next(&mut self.0.borrow_mut()) {
            Ok(None) => None,
            Ok(Some(token)) => Some(Ok((0, token, 0))),
            Err(error) => Some(Err(error)),
        }
    }
}

fn cmd_parse_do_file(
    f: Vec<u8>,
    pi: &mut cmd_parse_input,
    cause: &mut Option<std::ffi::CString>,
) -> Option<Box<cmd_parse_commands>> {
    let lexer = cmd_parse_state {
        f: Some(f),
        input: Some(pi),
        ..Default::default()
    };
    cmd_parse_run_parser(std::rc::Rc::new(std::cell::RefCell::new(lexer)), cause)
}
fn cmd_parse_do_buffer(
    buf: &[u8],
    pi: &mut cmd_parse_input,
    cause: &mut Option<std::ffi::CString>,
) -> Option<Box<cmd_parse_commands>> {
    let lexer = cmd_parse_state {
        buf: Some(buf),
        len: buf.len(),
        input: Some(pi),
        ..Default::default()
    };
    cmd_parse_run_parser(std::rc::Rc::new(std::cell::RefCell::new(lexer)), cause)
}
unsafe fn cmd_parse_log_commands(cmds: &mut cmd_parse_commands, prefix: &CStr) {
    unsafe {
        let mut i: u_int;
        let mut j: u_int;
        i = 0 as u_int;
        for cmd in command_list(cmds) {
            j = 0 as u_int;
            for arg in argument_list(&mut cmd.arguments) {
                match arg.type_0 {
                    CMD_PARSE_STRING => {
                        log_debug(
                            c"%s %u:%u: %s",
                            fmt_args![prefix, i, j, arg.string.as_ref().unwrap().as_c_str()],
                        );
                    }
                    CMD_PARSE_COMMANDS => {
                        let s = xasprintf(c"%s %u:%u", fmt_args![prefix, i, j]);
                        if let Some(commands) = arg.commands_ptr() {
                            cmd_parse_log_commands(commands, &s);
                        }
                    }
                    CMD_PARSE_PARSED_COMMANDS => {
                        let s = arg
                            .cmdlist
                            .as_ref()
                            .map_or_else(std::ffi::CString::default, |list| list.print(0));
                        log_debug(c"%s %u:%u: %s", fmt_args![prefix, i, j, s.as_c_str()]);
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
    cmd: &mut cmd_parse_command,
    pi: &mut cmd_parse_input,
    pr: &mut cmd_parse_result,
) -> core::ffi::c_int {
    unsafe {
        let mut cause = None;
        if pi.flags & CMD_PARSE_NOALIAS != 0 {
            return 0 as core::ffi::c_int;
        }
        *pr = cmd_parse_result::default();
        let Some(first) = cmd.arguments.first_mut() else {
            pr.status = CMD_PARSE_SUCCESS;
            pr.cmdlist = Some(CmdListRef::empty());
            return 1 as core::ffi::c_int;
        };
        if first.type_0 != CMD_PARSE_STRING {
            pr.status = CMD_PARSE_SUCCESS;
            pr.cmdlist = Some(CmdListRef::empty());
            return 1 as core::ffi::c_int;
        }
        let name = first.string.as_ref().unwrap();
        let Some(alias) = cmd_get_alias(name) else {
            return 0 as core::ffi::c_int;
        };
        log_debug(
            c"%s: %u alias %s = %s",
            fmt_args![
                c"cmd_parse_expand_alias",
                pi.line,
                name.as_c_str(),
                alias.as_c_str()
            ],
        );
        let Some(mut cmds) = cmd_parse_do_buffer(alias.as_bytes(), pi, &mut cause) else {
            pr.status = CMD_PARSE_ERROR;
            pr.error = cause;
            return 1 as core::ffi::c_int;
        };
        let Some(last) = cmds.last_mut() else {
            pr.status = CMD_PARSE_SUCCESS;
            pr.cmdlist = Some(CmdListRef::empty());
            return 1 as core::ffi::c_int;
        };
        drop(cmd.arguments.remove(0));
        let moved = core::mem::take(&mut cmd.arguments);
        last.arguments.extend(moved);
        cmd_parse_log_commands(&mut cmds, c"cmd_parse_expand_alias");
        pi.flags |= CMD_PARSE_NOALIAS;
        cmd_parse_build_commands(&mut cmds, pi, pr);
        pi.flags &= !CMD_PARSE_NOALIAS;
        1 as core::ffi::c_int
    }
}
unsafe fn cmd_parse_build_command(
    cmd: &mut cmd_parse_command,
    pi: &mut cmd_parse_input,
    pr: &mut cmd_parse_result,
) {
    unsafe {
        let mut current_block: u64;
        let mut values = Vec::<ArgsValue>::new();
        *pr = cmd_parse_result::default();
        if cmd_parse_expand_alias(cmd, pi, pr) != 0 {
            return;
        }
        current_block = 5143058163439228106;
        for arg in argument_list(&mut cmd.arguments) {
            values.push(ArgsValue::default());
            let value = values.last_mut().unwrap();
            match arg.type_0 {
                CMD_PARSE_STRING => {
                    *value = ArgsValue::String(arg.string.as_ref().unwrap().clone());
                }
                CMD_PARSE_COMMANDS => {
                    let commands = arg
                        .commands_ptr()
                        .expect("a commands argument carries commands");
                    cmd_parse_build_commands(commands, pi, pr);
                    if pr.status as core::ffi::c_uint
                        != CMD_PARSE_SUCCESS as core::ffi::c_int as core::ffi::c_uint
                    {
                        current_block = 689484554684886290;
                        break;
                    }
                    *value = ArgsValue::Commands {
                        cmdlist: pr.cmdlist.clone(),
                        cached: std::cell::OnceCell::new(),
                    };
                }
                CMD_PARSE_PARSED_COMMANDS => {
                    *value = ArgsValue::Commands {
                        cmdlist: arg.cmdlist.clone(),
                        cached: std::cell::OnceCell::new(),
                    };
                }
                _ => {}
            }
        }
        if current_block == 5143058163439228106 {
            match cmd_parse(&values, pi.file(), pi.line, pi.flags) {
                Ok(add) => {
                    pr.status = CMD_PARSE_SUCCESS;
                    pr.cmdlist = Some(CmdListRef::empty());
                    (pr.cmdlist.as_ref().unwrap()).append(add);
                }
                Err(cause) => {
                    pr.status = CMD_PARSE_ERROR;
                    pr.error = Some(cmd_parse_get_error(pi.file(), pi.line, &cause));
                }
            }
        }
    }
}
unsafe fn cmd_parse_build_commands(
    cmds: &mut cmd_parse_commands,
    pi: &mut cmd_parse_input,
    pr: &mut cmd_parse_result,
) {
    unsafe {
        let mut line: u_int = UINT_MAX;
        let mut current: Option<CmdListRef> = None;
        let mut result: Option<CmdListRef>;
        *pr = cmd_parse_result::default();
        if (*cmds).is_empty() {
            pr.status = CMD_PARSE_SUCCESS;
            pr.cmdlist = Some(CmdListRef::empty());
            return;
        }
        cmd_parse_log_commands(cmds, c"cmd_parse_build_commands");
        result = Some(CmdListRef::empty());
        for cmd in command_list(cmds) {
            if !pi.flags & CMD_PARSE_ONEGROUP != 0 && cmd.line != line {
                if let Some(current_list) = current.as_ref() {
                    cmd_parse_print_commands(&*pi, current_list);
                    (result.as_ref().unwrap()).move_from(current_list);
                }
                current = Some(CmdListRef::empty());
            }
            if current.is_none() {
                current = Some(CmdListRef::empty());
            }
            pi.line = cmd.line;
            line = pi.line;
            cmd_parse_build_command(cmd, pi, pr);
            if pr.status as core::ffi::c_uint
                != CMD_PARSE_SUCCESS as core::ffi::c_int as core::ffi::c_uint
            {
                return;
            }
            (current.as_ref().unwrap()).append_all(pr.cmdlist.as_ref().unwrap());
            let _ = pr.cmdlist.take();
        }
        if let Some(current_list) = current.as_ref() {
            cmd_parse_print_commands(&*pi, current_list);
            (result.as_ref().unwrap()).move_from(current_list);
        }
        let s = (result.as_ref().unwrap()).print(0 as core::ffi::c_int);
        log_debug(
            c"%s: %s",
            fmt_args![c"cmd_parse_build_commands", s.as_c_str()],
        );
        pr.status = CMD_PARSE_SUCCESS;
        pr.cmdlist = result.take();
    }
}
pub(crate) unsafe fn cmd_parse_from_file_impl(
    f: Vec<u8>,
    pi: Option<&mut cmd_parse_input>,
) -> cmd_parse_result {
    unsafe {
        let mut input = cmd_parse_input::default();
        let mut cause = None;
        let pi = &mut *pi.unwrap_or(&mut input);
        let mut pr = cmd_parse_result::default();
        let Some(mut cmds) = cmd_parse_do_file(f, &mut *pi, &mut cause) else {
            pr.status = CMD_PARSE_ERROR;
            pr.error = cause;
            return pr;
        };
        cmd_parse_build_commands(&mut cmds, pi, &mut pr);
        pr
    }
}
pub(crate) unsafe fn cmd_parse_from_string_impl(
    s: &CStr,
    pi: Option<&mut cmd_parse_input>,
) -> cmd_parse_result {
    unsafe {
        let mut input = cmd_parse_input::default();
        let pi = &mut *pi.unwrap_or(&mut input);
        pi.flags |= CMD_PARSE_ONEGROUP;
        cmd_parse_from_buffer_impl(s.to_bytes(), Some(&mut *pi))
    }
}
pub(crate) unsafe fn cmd_parse_and_append(
    s: &CStr,
    pi: Option<&mut cmd_parse_input>,
    c: Option<&ClientRef>,
    state: &CmdqStateRef,
    error: &mut Option<std::ffi::CString>,
) -> cmd_parse_status {
    unsafe {
        let mut pr = cmd_parse_from_string(s, pi);
        match pr.status {
            CMD_PARSE_ERROR => {
                *error = pr.error.take();
            }
            CMD_PARSE_SUCCESS => {
                let cmdlist = pr.cmdlist.take().unwrap();
                cmdq_append(c, cmdlist.queue_items(Some(state)));
            }
            _ => {}
        }
        pr.status
    }
}
pub(crate) unsafe fn cmd_parse_from_buffer_impl(
    buf: &[u8],
    pi: Option<&mut cmd_parse_input>,
) -> cmd_parse_result {
    unsafe {
        let mut input = cmd_parse_input::default();
        let mut cause = None;
        let pi = &mut *pi.unwrap_or(&mut input);
        let mut pr = cmd_parse_result::default();
        if buf.is_empty() {
            pr.status = CMD_PARSE_SUCCESS;
            pr.cmdlist = Some(CmdListRef::empty());
            return pr;
        }
        let Some(mut cmds) = cmd_parse_do_buffer(buf, &mut *pi, &mut cause) else {
            pr.status = CMD_PARSE_ERROR;
            pr.error = cause;
            return pr;
        };
        cmd_parse_build_commands(&mut cmds, pi, &mut pr);
        pr
    }
}
pub(crate) unsafe fn cmd_parse_from_arguments_impl(
    values: &[ArgsValue],
    pi: Option<&mut cmd_parse_input>,
) -> cmd_parse_result {
    unsafe {
        let mut input = cmd_parse_input::default();
        let mut cmd: Box<cmd_parse_command>;
        let mut end: core::ffi::c_int;
        let pi = &mut *pi.unwrap_or(&mut input);
        let mut pr = cmd_parse_result::default();
        let mut cmds = Box::new(cmd_parse_commands::new());
        cmd = cmd_parse_new_command(pi.line);
        for val in values {
            end = 0 as core::ffi::c_int;
            if matches!(val, ArgsValue::String(_)) {
                let mut copy = val.string().to_bytes().to_vec();
                let mut size = copy.len();
                if size != 0 && copy[size - 1] as core::ffi::c_int == ';' as i32 {
                    size -= 1;
                    copy.truncate(size);
                    if size > 0 && copy[size - 1] as core::ffi::c_int == '\\' as i32 {
                        copy[size - 1] = b';';
                    } else {
                        end = 1 as core::ffi::c_int;
                    }
                }
                if end == 0 || size != 0 {
                    let mut arg = cmd_parse_new_argument();
                    arg.type_0 = CMD_PARSE_STRING;
                    arg.string =
                        Some(std::ffi::CString::new(copy).expect("command argument has no NUL"));
                    cmd.arguments.push(arg);
                } else {
                    drop(copy);
                }
            } else if let ArgsValue::Commands { cmdlist, .. } = val {
                let mut arg = cmd_parse_new_argument();
                arg.type_0 = CMD_PARSE_PARSED_COMMANDS;
                arg.cmdlist = cmdlist.clone();
                cmd.arguments.push(arg);
            } else {
                fatalx(c"unknown argument type", fmt_args![]);
            }
            if end != 0 {
                cmds.push(cmd);
                cmd = cmd_parse_new_command(pi.line);
            }
        }
        if !cmd.arguments.is_empty() {
            cmds.push(cmd);
        } else {
            drop(cmd);
        }
        cmd_parse_build_commands(&mut cmds, pi, &mut pr);
        pr
    }
}

pub unsafe fn cmd_parse_from_file(
    file: Vec<u8>,
    input: Option<&mut cmd_parse_input>,
) -> cmd_parse_result {
    unsafe { RustCommandParser.parse_file(file, input) }
}

pub unsafe fn cmd_parse_from_buffer(
    buffer: &[u8],
    input: Option<&mut cmd_parse_input>,
) -> cmd_parse_result {
    unsafe { RustCommandParser.parse_buffer(buffer, input) }
}

pub unsafe fn cmd_parse_from_string(
    string: &CStr,
    input: Option<&mut cmd_parse_input>,
) -> cmd_parse_result {
    unsafe { RustCommandParser.parse_string(string, input) }
}

pub unsafe fn cmd_parse_from_arguments(
    arguments: &[ArgsValue],
    input: Option<&mut cmd_parse_input>,
) -> cmd_parse_result {
    unsafe { RustCommandParser.parse_arguments(arguments, input) }
}

fn yyerror(ps: &mut cmd_parse_state<'_>, fmt: &CStr, args: &[FmtArg]) {
    let pi = ps.input();
    if ps.error.is_some() {
        return;
    }
    let error = format_alloc(fmt, args);
    ps.error = Some(cmd_parse_get_error(pi.file(), pi.line, &error));
}
fn yylex_is_var(ch: core::ffi::c_char, first: core::ffi::c_int) -> core::ffi::c_int {
    let ch = ch as u8 as core::ffi::c_int;
    unsafe {
        (ch != b'=' as core::ffi::c_int
            && (first == 0 || libc::isdigit(ch) == 0)
            && (libc::isalnum(ch) != 0 || ch == b'_' as core::ffi::c_int))
            as core::ffi::c_int
    }
}
/// The collected bytes as a C string, which ends at the first NUL the same
/// way reading the lexer's NUL-terminated buffer did.
fn yylex_cstring(mut buf: Vec<u8>) -> std::ffi::CString {
    if let Some(nul) = buf.iter().position(|&byte| byte == 0) {
        buf.truncate(nul);
    }
    std::ffi::CString::new(buf).expect("lexer bytes were truncated at the first NUL")
}
fn yylex_getc1(ps: &mut cmd_parse_state<'_>) -> core::ffi::c_int {
    let ch: core::ffi::c_int;
    if let Some(file) = ps.f.as_ref() {
        let off = ps.off;
        if off == file.len() {
            ch = EOF;
        } else {
            ch = file[off] as core::ffi::c_int;
            ps.off = off.wrapping_add(1);
        }
    } else if ps.off == ps.len {
        ch = EOF;
    } else {
        let fresh27 = ps.off;
        ps.off = ps.off.wrapping_add(1);
        ch = ps.buf.expect("buffer parser state")[fresh27] as core::ffi::c_int;
    }
    ch
}
fn yylex_ungetc(ps: &mut cmd_parse_state<'_>, ch: core::ffi::c_int) {
    if ps.off > 0 as size_t && ch != EOF {
        ps.off = ps.off.wrapping_sub(1);
    }
}
fn yylex_getc(ps: &mut cmd_parse_state<'_>) -> core::ffi::c_int {
    let mut ch: core::ffi::c_int;
    if ps.escapes != 0 as u_int {
        ps.escapes = ps.escapes.wrapping_sub(1);
        return '\\' as i32;
    }
    loop {
        ch = yylex_getc1(ps);
        if ch == '\\' as i32 {
            ps.escapes = ps.escapes.wrapping_add(1);
        } else if ch == '\n' as i32 && ps.escapes.wrapping_rem(2 as u_int) == 1 as u_int {
            ps.advance_line();
            ps.escapes = ps.escapes.wrapping_sub(1);
        } else {
            if ps.escapes != 0 as u_int {
                yylex_ungetc(ps, ch);
                ps.escapes = ps.escapes.wrapping_sub(1);
                return '\\' as i32;
            }
            return ch;
        }
    }
}
fn yylex_get_word(ps: &mut cmd_parse_state<'_>, mut ch: core::ffi::c_int) -> std::ffi::CString {
    {
        let mut buf: Vec<u8> = Vec::new();
        loop {
            buf.push(ch as u8);
            ch = yylex_getc(ps);
            if !(ch != EOF && !c" \t\n".to_bytes_with_nul().contains(&(ch as u8))) {
                break;
            }
        }
        yylex_ungetc(ps, ch);
        let word = yylex_cstring(buf);
        log_debug(c"%s: %s", fmt_args![c"yylex_get_word", word.as_c_str()]);
        word
    }
}
fn yylex_next(ps: &mut cmd_parse_state<'_>) -> Result<Option<Token>, LexError> {
    unsafe {
        let mut ch: core::ffi::c_int;
        let mut next: core::ffi::c_int;
        if ps.eol != 0 {
            ps.advance_line();
        }
        ps.eol = 0 as core::ffi::c_int;
        let condition = ps.condition;
        ps.condition = 0 as core::ffi::c_int;
        loop {
            ch = yylex_getc(ps);
            if ch == EOF {
                if ps.eof != 0 {
                    return Ok(None);
                }
                ps.eof = 1 as core::ffi::c_int;
                return Ok(Some(Token::Newline));
            }
            if ch == ' ' as i32 || ch == '\t' as i32 {
                continue;
            }
            if ch == '\r' as i32 {
                ch = yylex_getc(ps);
                if ch != '\n' as i32 {
                    yylex_ungetc(ps, ch);
                    ch = '\r' as i32;
                }
            }
            if ch == '\n' as i32 {
                ps.eol = 1 as core::ffi::c_int;
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
                next = yylex_getc(ps);
                if condition != 0 && next == '{' as i32 {
                    let Some(token) = yylex_format(ps) else {
                        return Err(LexError);
                    };
                    return Ok(Some(Token::Format(TokenText::from_cstring(token))));
                }
                while next != '\n' as i32 && next != EOF {
                    next = yylex_getc(ps);
                }
                if next == '\n' as i32 {
                    ps.advance_line();
                    return Ok(Some(Token::Newline));
                }
                continue;
            }
            if ch == '%' as i32 {
                let word = TokenText::from_cstring(yylex_get_word(ps, '%' as i32));
                if word
                    .as_c_str()
                    .to_bytes()
                    .iter()
                    .all(|&byte| byte == b'%' || libc::isdigit(byte as core::ffi::c_int) != 0)
                {
                    return Ok(Some(Token::Word(word)));
                }
                ps.condition = 1;
                return match word.as_c_str().to_bytes() {
                    b"%hidden" => Ok(Some(Token::Hidden)),
                    b"%if" => Ok(Some(Token::If)),
                    b"%else" => Ok(Some(Token::Else)),
                    b"%elif" => Ok(Some(Token::Elif)),
                    b"%endif" => Ok(Some(Token::Endif)),
                    _ => Err(LexError),
                };
            }
            let Some(token) = yylex_token(ps, ch) else {
                return Err(LexError);
            };
            let token = TokenText::from_cstring(token);
            let bytes = token.as_c_str().to_bytes();
            if let Some(equals) = bytes.iter().position(|&byte| byte == b'=')
                && equals != 0
                && bytes[..equals].iter().enumerate().all(|(at, &byte)| {
                    yylex_is_var(byte as core::ffi::c_char, (at == 0) as core::ffi::c_int) != 0
                })
            {
                return Ok(Some(Token::Equals(token)));
            }
            return Ok(Some(Token::Word(token)));
        }
    }
}
fn yylex_format(ps: &mut cmd_parse_state<'_>) -> Option<std::ffi::CString> {
    {
        let current_block: u64;
        let mut buf: Vec<u8> = Vec::new();
        let mut ch: core::ffi::c_int;
        let mut brackets: core::ffi::c_int = 1 as core::ffi::c_int;
        buf.extend_from_slice(b"#{");
        loop {
            ch = yylex_getc(ps);
            if ch == EOF || ch == '\n' as i32 {
                current_block = 13016994178946890092;
                break;
            }
            if ch == '#' as i32 {
                ch = yylex_getc(ps);
                if ch == EOF || ch == '\n' as i32 {
                    current_block = 13016994178946890092;
                    break;
                }
                if ch == '{' as i32 {
                    brackets += 1;
                }
                buf.push(b'#');
            } else if ch == '}' as i32 && brackets != 0 as core::ffi::c_int && {
                brackets -= 1;
                brackets == 0 as core::ffi::c_int
            } {
                buf.push(ch as u8);
                current_block = 10048703153582371463;
                break;
            }
            buf.push(ch as u8);
        }
        match current_block {
            10048703153582371463 if !(brackets != 0 as core::ffi::c_int) => {
                let token = yylex_cstring(buf);
                log_debug(c"%s: %s", fmt_args![c"yylex_format", token.as_c_str()]);
                return Some(token);
            }
            _ => {}
        }
        None
    }
}
unsafe fn yylex_token_escape(ps: &mut cmd_parse_state<'_>, buf: &mut Vec<u8>) -> core::ffi::c_int {
    unsafe {
        let current_block: u64;
        let mut ch: core::ffi::c_int;
        let mut type_0: core::ffi::c_int = 0;
        let o2: core::ffi::c_int;
        let o3: core::ffi::c_int;
        let mlen: core::ffi::c_int;
        let mut size: u_int = 0;
        let mut i: u_int;
        let mut tmp: u_int = 0;
        let mut m: [core::ffi::c_char; 16] = [0; 16];
        ch = yylex_getc(ps);
        if ch >= '4' as i32 && ch <= '7' as i32 {
            yyerror(ps, c"invalid octal escape", fmt_args![]);
            return 0 as core::ffi::c_int;
        }
        if ch >= '0' as i32 && ch <= '3' as i32 {
            o2 = yylex_getc(ps);
            if o2 >= '0' as i32 && o2 <= '7' as i32 {
                o3 = yylex_getc(ps);
                if o3 >= '0' as i32 && o3 <= '7' as i32 {
                    ch = 64 as core::ffi::c_int * (ch - '0' as i32)
                        + 8 as core::ffi::c_int * (o2 - '0' as i32)
                        + (o3 - '0' as i32);
                    buf.push(ch as u8);
                    return 1 as core::ffi::c_int;
                }
            }
            yyerror(ps, c"invalid octal escape", fmt_args![]);
            return 0 as core::ffi::c_int;
        }
        match ch {
            EOF => return 0 as core::ffi::c_int,
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
                    ch = yylex_getc(ps);
                    if ch == EOF || ch == '\n' as i32 {
                        return 0 as core::ffi::c_int;
                    }
                    let Some(digit) = (ch as u8 as char).to_digit(16) else {
                        yyerror(ps, c"invalid \\%c argument", fmt_args![type_0]);
                        return 0;
                    };
                    tmp = (tmp << 4) | digit;
                    i = i.wrapping_add(1);
                }
                mlen = wctomb(m.as_mut_ptr(), tmp as wchar_t);
                if mlen <= 0 as core::ffi::c_int
                    || mlen > size_of::<[core::ffi::c_char; 16]>() as core::ffi::c_int
                {
                    yyerror(ps, c"invalid \\%c argument", fmt_args![type_0]);
                    return 0 as core::ffi::c_int;
                }
                buf.extend(m[..mlen as usize].iter().map(|&byte| byte as u8));
                1 as core::ffi::c_int
            }
            _ => {
                buf.push(ch as u8);
                1 as core::ffi::c_int
            }
        }
    }
}
unsafe fn yylex_token_variable(
    ps: &mut cmd_parse_state<'_>,
    buf: &mut Vec<u8>,
) -> core::ffi::c_int {
    {
        let mut ch: core::ffi::c_int;
        let mut brackets: core::ffi::c_int = 0 as core::ffi::c_int;
        let mut name: [u8; 1024] = [0; 1024];
        let mut namelen: size_t = 0 as size_t;
        ch = yylex_getc(ps);
        if ch == EOF {
            return 0 as core::ffi::c_int;
        }
        if ch == '{' as i32 {
            brackets = 1 as core::ffi::c_int;
        } else {
            if yylex_is_var(ch as core::ffi::c_char, 1 as core::ffi::c_int) == 0 {
                buf.push(b'$');
                yylex_ungetc(ps, ch);
                return 1 as core::ffi::c_int;
            }
            let fresh28 = namelen;
            namelen = namelen.wrapping_add(1);
            name[fresh28 as usize] = ch as u8;
        }
        loop {
            ch = yylex_getc(ps);
            if brackets != 0 && ch == '}' as i32 {
                break;
            }
            if ch == EOF || yylex_is_var(ch as core::ffi::c_char, 0 as core::ffi::c_int) == 0 {
                if brackets == 0 {
                    yylex_ungetc(ps, ch);
                    break;
                } else {
                    yyerror(ps, c"invalid environment variable", fmt_args![]);
                    return 0 as core::ffi::c_int;
                }
            } else {
                if namelen == name.len() - 2 {
                    yyerror(ps, c"environment variable is too long", fmt_args![]);
                    return 0 as core::ffi::c_int;
                }
                let fresh29 = namelen;
                namelen = namelen.wrapping_add(1);
                name[fresh29 as usize] = ch as u8;
            }
        }
        let name = CStr::from_bytes_with_nul(&name[..=namelen]).expect("terminated lexer name");
        let value = with_global_environment(|env| {
            env.find(name)
                .and_then(|entry| entry.value.map(CStr::to_owned))
        });
        if let Some(value) = value {
            log_debug(
                c"%s: %s -> %s",
                fmt_args![c"yylex_token_variable", name, value.as_c_str()],
            );
            buf.extend_from_slice(value.to_bytes());
        }
        1 as core::ffi::c_int
    }
}
unsafe fn yylex_token_tilde(ps: &mut cmd_parse_state<'_>, buf: &mut Vec<u8>) -> core::ffi::c_int {
    unsafe {
        let mut ch: core::ffi::c_int;
        let mut name: [u8; 1024] = [0; 1024];
        let mut namelen: size_t = 0 as size_t;
        loop {
            ch = yylex_getc(ps);
            if ch == EOF || c"/ \t\n\"'".to_bytes_with_nul().contains(&(ch as u8)) {
                yylex_ungetc(ps, ch);
                break;
            } else {
                if namelen == name.len() - 2 {
                    yyerror(ps, c"user name is too long", fmt_args![]);
                    return 0 as core::ffi::c_int;
                }
                let fresh30 = namelen;
                namelen = namelen.wrapping_add(1);
                name[fresh30 as usize] = ch as u8;
            }
        }
        let name = CStr::from_bytes_with_nul(&name[..=namelen]).expect("terminated lexer name");
        let global_home = if name.is_empty() {
            with_global_environment(|env| {
                env.find(c"HOME")
                    .and_then(|entry| entry.value.map(CStr::to_owned))
            })
        } else {
            None
        };
        let home = if name.is_empty() {
            global_home.filter(|value| !value.is_empty()).or_else(|| {
                UserAccountRecord::lookup_uid(getuid())?
                    .account_home()
                    .map(CStr::to_owned)
            })
        } else {
            UserAccountRecord::lookup_name(name)
                .and_then(|account| account.account_home().map(CStr::to_owned))
        };
        let Some(home) = home else {
            return 0 as core::ffi::c_int;
        };
        log_debug(
            c"%s: ~%s -> %s",
            fmt_args![c"yylex_token_tilde", name, home.as_c_str()],
        );
        buf.extend_from_slice(home.to_bytes());
        1 as core::ffi::c_int
    }
}
fn yylex_token(
    ps: &mut cmd_parse_state<'_>,
    mut ch: core::ffi::c_int,
) -> Option<std::ffi::CString> {
    unsafe {
        let mut current_block: u64;
        let mut buf: Vec<u8> = Vec::new();
        let mut state: cmd_parse_token_state = NONE;
        let mut last: cmd_parse_token_state = START;
        loop {
            if ch == EOF {
                log_debug(c"%s: end at EOF", fmt_args![c"yylex_token"]);
                current_block = 13321564401369230990;
                break;
            } else {
                if state as core::ffi::c_uint == NONE as core::ffi::c_int as core::ffi::c_uint
                    && ch == '\r' as i32
                {
                    ch = yylex_getc(ps);
                    if ch != '\n' as i32 {
                        yylex_ungetc(ps, ch);
                        ch = '\r' as i32;
                    }
                }
                if ch == '\n' as i32 {
                    if state as core::ffi::c_uint == NONE as core::ffi::c_int as core::ffi::c_uint {
                        log_debug(c"%s: end at EOL", fmt_args![c"yylex_token"]);
                        current_block = 13321564401369230990;
                        break;
                    } else {
                        ps.advance_line();
                    }
                }
                if state as core::ffi::c_uint == NONE as core::ffi::c_int as core::ffi::c_uint
                    && (ch == ' ' as i32 || ch == '\t' as i32)
                {
                    log_debug(c"%s: end at WS", fmt_args![c"yylex_token"]);
                    current_block = 13321564401369230990;
                    break;
                } else if state as core::ffi::c_uint
                    == NONE as core::ffi::c_int as core::ffi::c_uint
                    && (ch == ';' as i32 || ch == '}' as i32)
                {
                    log_debug(c"%s: end at %c", fmt_args![c"yylex_token", ch]);
                    current_block = 13321564401369230990;
                    break;
                } else if ch == '\n' as i32
                    && state as core::ffi::c_uint != NONE as core::ffi::c_int as core::ffi::c_uint
                {
                    buf.push(b'\n');
                    loop {
                        ch = yylex_getc(ps);
                        if !(ch == ' ' as i32 || ch == '\t' as i32) {
                            break;
                        }
                    }
                    if ch != '#' as i32 {
                        continue;
                    }
                    ch = yylex_getc(ps);
                    if c",#{}:".to_bytes_with_nul().contains(&(ch as u8)) {
                        yylex_ungetc(ps, ch);
                        ch = '#' as i32;
                    } else {
                        loop {
                            ch = yylex_getc(ps);
                            if !(ch != '\n' as i32 && ch != EOF) {
                                break;
                            }
                        }
                    }
                } else {
                    if ch == '\\' as i32
                        && state as core::ffi::c_uint
                            != SINGLE_QUOTES as core::ffi::c_int as core::ffi::c_uint
                    {
                        if yylex_token_escape(ps, &mut buf) == 0 {
                            current_block = 11768010348333939680;
                            break;
                        }
                        current_block = 9512337080773452662;
                    } else if ch == '~' as i32
                        && last as core::ffi::c_uint != state as core::ffi::c_uint
                        && state as core::ffi::c_uint
                            != SINGLE_QUOTES as core::ffi::c_int as core::ffi::c_uint
                    {
                        if yylex_token_tilde(ps, &mut buf) == 0 {
                            current_block = 11768010348333939680;
                            break;
                        }
                        current_block = 9512337080773452662;
                    } else if ch == '$' as i32
                        && state as core::ffi::c_uint
                            != SINGLE_QUOTES as core::ffi::c_int as core::ffi::c_uint
                    {
                        if yylex_token_variable(ps, &mut buf) == 0 {
                            current_block = 11768010348333939680;
                            break;
                        }
                        current_block = 9512337080773452662;
                    } else {
                        if ch == '}' as i32
                            && state as core::ffi::c_uint
                                == NONE as core::ffi::c_int as core::ffi::c_uint
                        {
                            current_block = 11768010348333939680;
                            break;
                        }
                        if ch == '\'' as i32 {
                            if state as core::ffi::c_uint
                                == NONE as core::ffi::c_int as core::ffi::c_uint
                            {
                                state = SINGLE_QUOTES;
                                current_block = 12867991516770085914;
                            } else if state as core::ffi::c_uint
                                == SINGLE_QUOTES as core::ffi::c_int as core::ffi::c_uint
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
                                    if state as core::ffi::c_uint
                                        == NONE as core::ffi::c_int as core::ffi::c_uint
                                    {
                                        state = DOUBLE_QUOTES;
                                        current_block = 12867991516770085914;
                                    } else if state as core::ffi::c_uint
                                        == DOUBLE_QUOTES as core::ffi::c_int as core::ffi::c_uint
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
                    ch = yylex_getc(ps);
                }
            }
        }
        match current_block {
            11768010348333939680 => None,
            _ => {
                yylex_ungetc(ps, ch);
                let token = yylex_cstring(buf);
                log_debug(c"%s: %s", fmt_args![c"yylex_token", token.as_c_str()]);
                Some(token)
            }
        }
    }
}

#[cfg(test)]
#[path = "../tests/test_cmd_parse_focused.rs"]
mod focused_tests;
