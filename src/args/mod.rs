pub mod argument_command_state;
pub mod argument_entry;
pub mod argument_parse_spec;
pub mod argument_text;
pub mod argument_value;
pub mod arguments_trait;

use crate::cmd::{CmdListRef, cmd};
use crate::cmd::cmdq_item;
use crate::cmd::cmd_find_copy_state;
use crate::cmd::cmd_parse_from_string;
use crate::cmd::cmdq_item_ref_of;
use crate::cmd::{cmd_get_args, cmd_get_entry, cmd_get_source};
use crate::cmd::{cmd_log_argv, cmd_template_replace};
use crate::compat::strtonum;
use crate::fmt_args;
use crate::format::format_single_from_target;
use crate::log::{fatalx, log_debug};
use crate::text::{RustUtf8VisModel, Utf8VisModel};
pub use crate::types::*;
use crate::xmalloc::xasprintf;
use ::std::ffi::CStr;
use ::std::ffi::CString;
use core::ffi::c_int;
#[repr(C)]
#[derive(Default)]
pub struct args {
    tree: args_tree,
    count: u_int,
    values: Vec<args_value_t>,
}

/// Parsed-argument state backed by hmux's transpiled argument structure.
#[repr(transparent)]
#[derive(Default)]
pub struct RustArguments(args);

impl RustArguments {
    /// Borrows existing arguments without allocating or transferring ownership.
    pub fn from_ref(arguments: &args) -> &Self {
        unsafe { &*(arguments as *const args).cast::<Self>() }
    }

    /// Mutably borrows existing arguments without allocating or transferring ownership.
    pub fn from_mut(arguments: &mut args) -> &mut Self {
        unsafe { &mut *(arguments as *mut args).cast::<Self>() }
    }

    /// Wraps existing argument state by value.
    pub fn from_args(arguments: args) -> Self {
        Self(arguments)
    }

    /// Returns the argument state for an existing args-based API.
    pub fn into_args(self) -> args {
        self.0
    }

    /// Borrows the opaque state for an existing args-based API.
    pub fn as_args(&self) -> &args {
        &self.0
    }

    /// Mutably borrows the opaque state for an existing args-based API.
    pub fn as_args_mut(&mut self) -> &mut args {
        &mut self.0
    }

    /// Builds positional arguments from strings.
    pub fn from_strings(values: &[&CStr]) -> Self {
        let mut arguments = Self::default();
        for value in values {
            arguments.0.values.push(args_value_t {
                value: ArgsValue::String((*value).to_owned()),
            });
            arguments.0.count += 1;
        }
        arguments
    }

    /// Returns one positional value.
    pub fn argument_value(&self, index: u_int) -> Option<&args_value_t> {
        self.argument_values().get(index as usize)
    }

    pub fn argument_flag_count(&self, flag: u_char) -> c_int {
        args_has(&self.0, flag)
    }

    pub fn set_argument_flag(&mut self, flag: u_char, value: Option<Box<args_value_t>>, flags: c_int) {
        unsafe { args_set(&mut self.0, flag, value, flags) }
    }

    pub fn argument_flag_string(&self, flag: u_char) -> Option<&CStr> {
        args_get_str(&self.0, flag)
    }

    pub fn argument_flags(&self) -> Vec<u_char> {
        args_flags(&self.0).collect()
    }

    pub fn argument_count(&self) -> u_int {
        args_count(&self.0)
    }

    pub fn argument_values(&self) -> &[args_value_t] {
        &self.0.values
    }

    pub fn set_argument_string(&mut self, index: u_int, value: &CStr) -> bool {
        let Some(argument) = self.0.values.get_mut(index as usize) else {
            return false;
        };
        argument.value = ArgsValue::String(value.to_owned());
        true
    }

    pub fn argument_string(&self, index: u_int) -> Option<&CStr> {
        unsafe { args_string_str(&self.0, index) }
    }

    pub fn argument_flag_values(&self, flag: u_char) -> Vec<&args_value_t> {
        args_value_list(&self.0, flag)
    }
}

impl crate::Arguments for RustArguments {
    fn argument_flag_count(&self, flag: u_char) -> c_int {
        RustArguments::argument_flag_count(self, flag)
    }

    fn set_argument_flag(&mut self, flag: u_char, value: Option<Box<args_value_t>>, flags: c_int) {
        RustArguments::set_argument_flag(self, flag, value, flags)
    }

    fn argument_flag_string(&self, flag: u_char) -> Option<&CStr> {
        RustArguments::argument_flag_string(self, flag)
    }

    fn argument_flags(&self) -> Vec<u_char> {
        RustArguments::argument_flags(self)
    }

    fn argument_count(&self) -> u_int {
        RustArguments::argument_count(self)
    }

    fn argument_values(&self) -> &[args_value_t] {
        RustArguments::argument_values(self)
    }

    fn set_argument_string(&mut self, index: u_int, value: &CStr) -> bool {
        RustArguments::set_argument_string(self, index, value)
    }

    fn argument_string(&self, index: u_int) -> Option<&CStr> {
        RustArguments::argument_string(self, index)
    }

    fn argument_flag_values(&self, flag: u_char) -> Vec<&args_value_t> {
        RustArguments::argument_flag_values(self, flag)
    }
}

impl crate::Arguments for args {
    fn argument_flag_count(&self, flag: u_char) -> c_int {
        RustArguments::argument_flag_count(RustArguments::from_ref(self), flag)
    }

    fn set_argument_flag(&mut self, flag: u_char, value: Option<Box<args_value_t>>, flags: c_int) {
        RustArguments::set_argument_flag(RustArguments::from_mut(self), flag, value, flags)
    }

    fn argument_flag_string(&self, flag: u_char) -> Option<&CStr> {
        RustArguments::argument_flag_string(RustArguments::from_ref(self), flag)
    }

    fn argument_flags(&self) -> Vec<u_char> {
        RustArguments::argument_flags(RustArguments::from_ref(self))
    }

    fn argument_count(&self) -> u_int {
        RustArguments::argument_count(RustArguments::from_ref(self))
    }

    fn argument_values(&self) -> &[args_value_t] {
        RustArguments::argument_values(RustArguments::from_ref(self))
    }

    fn set_argument_string(&mut self, index: u_int, value: &CStr) -> bool {
        RustArguments::set_argument_string(RustArguments::from_mut(self), index, value)
    }

    fn argument_string(&self, index: u_int) -> Option<&CStr> {
        RustArguments::argument_string(RustArguments::from_ref(self), index)
    }

    fn argument_flag_values(&self, flag: u_char) -> Vec<&args_value_t> {
        RustArguments::argument_flag_values(RustArguments::from_ref(self), flag)
    }
}

#[repr(C)]
#[derive(Default)]
pub struct args_command_state {
    pub(crate) cmdlist: Option<CmdListRef>,
    pub cmd: Option<CString>,
    pub pi: cmd_parse_input,
    pub(crate) source_file: Option<CString>,
    pub(crate) client_ref: Option<ClientRef>,
}

impl crate::ArgumentCommandState for args_command_state {
    fn prepared_command_list(&self) -> Option<&CmdListRef> {
        self.cmdlist.as_ref()
    }
    fn set_prepared_command_list(&mut self, command_list: Option<CmdListRef>) {
        self.cmdlist = command_list;
    }
    fn take_prepared_command_list(&mut self) -> Option<CmdListRef> {
        self.cmdlist.take()
    }
    fn prepared_command_text(&self) -> Option<&CStr> {
        self.cmd.as_deref()
    }
    fn set_prepared_command_text(&mut self, command: Option<CString>) {
        self.cmd = command;
    }
    fn prepared_command_parse_input(&self) -> &cmd_parse_input {
        &self.pi
    }
    fn prepared_command_parse_input_mut(&mut self) -> &mut cmd_parse_input {
        &mut self.pi
    }
    fn set_prepared_command_source(&mut self, file: Option<&CStr>, line: u_int) {
        self.source_file = file.map(CStr::to_owned);
        self.pi.file = self.source_file.clone();
        self.pi.line = line;
    }
    fn set_prepared_command_client(&mut self, client: Option<ClientRef>) {
        crate::CommandParseInput::set_command_parse_client(&mut self.pi, client.clone());
        self.client_ref = client;
    }
    fn prepared_command_client(&self) -> Option<&ClientRef> {
        self.client_ref.as_ref()
    }
}
impl Clone for args_command_state {
    fn clone(&self) -> Self {
        let source_file = self.source_file.clone();
        let mut pi = self.pi.clone();
        pi.file = source_file.clone();
        Self {
            cmdlist: self.cmdlist.clone(),
            cmd: self.cmd.clone(),
            pi,
            source_file,
            client_ref: self.client_ref.clone(),
        }
    }
}
pub use crate::consts::{
    ARGS_PARSE_COMMANDS, ARGS_PARSE_COMMANDS_OR_STRING, ARGS_PARSE_INVALID, ARGS_PARSE_STRING,
    CMD_PARSE_ERROR, CMD_PARSE_SUCCESS, RB_BLACK, RB_NEGINF, RB_RED, VIS_CSTYLE, VIS_DQ, VIS_NL,
    VIS_OCTAL, VIS_TAB,
};

pub const ARGS_ENTRY_OPTIONAL_VALUE: core::ffi::c_int = 0x1 as core::ffi::c_int;

fn value_at(values: &[args_value_t], i: usize) -> Option<&args_value_t> {
    values.get(i)
}

/// The entry a flag has in the arguments, if it has one.
fn args_find(args: &args, flag: u_char) -> Option<&args_entry> {
    args.tree.get(&flag).map(|entry| entry.as_ref())
}

/// The last value given for a flag, if it was given any.
fn args_last_value(args: &args, flag: u_char) -> Option<&args_value_t> {
    Some(args_find(args, flag)?.values.last()?.as_ref())
}

/// The string of the last value given for a flag, when there is one and it is
/// a string.
fn args_last_string(args: &args, flag: u_char) -> Option<&CStr> {
    let entry = args.tree.get(&flag)?;
    let value = entry.values.last()?;
    let ArgsValue::String(string) = &value.value else {
        return None;
    };
    Some(string)
}

/// Whether `b` is `isalnum` under the process's current locale, which is what
/// a flag has to be.
fn is_flag_byte(b: u8) -> bool {
    unsafe { libc::isalnum(b.into()) != 0 }
}

unsafe fn args_copy_value(to: &mut args_value_t, from: &args_value_t) {
    to.value = match &from.value {
        ArgsValue::Commands { cmdlist, .. } => ArgsValue::Commands {
            cmdlist: cmdlist.clone(),
            cached: std::cell::OnceCell::new(),
        },
        ArgsValue::String(string) => ArgsValue::String(string.clone()),
        ArgsValue::None => ArgsValue::None,
    };
}

fn args_value_type_to_string(value: &ArgsValue) -> &'static CStr {
    match value {
        ArgsValue::None => c"NONE",
        ArgsValue::String(_) => c"STRING",
        ArgsValue::Commands { .. } => c"COMMANDS",
    }
}

unsafe fn args_value_as_string(value: &args_value_t) -> &CStr {
    unsafe {
        match &value.value {
            ArgsValue::None => c"",
            ArgsValue::Commands { cmdlist, cached } => cached
                .get_or_init(|| {
                    cmdlist
                        .as_ref()
                        .map_or_else(CString::default, |list| list.print(0))
                })
                .as_c_str(),
            ArgsValue::String(string) => string.as_c_str(),
        }
    }
}

pub fn args_create() -> Box<args> {
    Box::default()
}

/// How the template says a flag takes its argument.
#[derive(Clone, Copy, PartialEq)]
enum FlagArgument {
    None,
    Required,
    Optional,
}

/// What the template says about a flag, or `None` when it has no such flag.
fn flag_in_template(template: &[u8], flag: u8) -> Option<FlagArgument> {
    let at = template.iter().position(|&b| b == flag)?;
    Some(match (template.get(at + 1), template.get(at + 2)) {
        (Some(b':'), Some(b':')) => FlagArgument::Optional,
        (Some(b':'), _) => FlagArgument::Required,
        _ => FlagArgument::None,
    })
}

/// Where reading the flags out of one word of a command line stopped.
enum Flags {
    /// The word was flags; the next word may hold more.
    More,
    /// The word is not a flag word: the arguments start here.
    Arguments,
    /// The word was rejected, with the reason in `cause` unless it was `-?`.
    Failed,
}

/// Takes the argument of a flag from the rest of its word, or from the word
/// after it when the flag ends the word.
unsafe fn args_parse_flag_argument(
    values: &[args_value_t],
    i: &mut usize,
    args: &mut args,
    cause: &mut Option<CString>,
    rest: &CStr,
    flag: u_char,
    optional: bool,
) -> Flags {
    unsafe {
        let mut new = Box::new(args_value_t::default());
        if !rest.is_empty() {
            new.value = ArgsValue::String(rest.to_owned());
        } else {
            let argument = value_at(values, *i);
            if argument.is_some_and(|argument| !matches!(&argument.value, ArgsValue::String(_))) {
                *cause = Some(xasprintf(
                    c"-%c argument must be a string",
                    fmt_args![flag as core::ffi::c_int],
                ));
                drop(new);
                return Flags::Failed;
            }
            let Some(argument) = argument else {
                drop(new);
                if optional {
                    log_debug(
                        c"%s: -%c (optional)",
                        fmt_args![
                            c"args_parse_flag_argument".as_ptr(),
                            flag as core::ffi::c_int
                        ],
                    );
                    args_set(args, flag, None, ARGS_ENTRY_OPTIONAL_VALUE);
                    return Flags::More;
                }
                *cause = Some(xasprintf(
                    c"-%c expects an argument",
                    fmt_args![flag as core::ffi::c_int],
                ));
                return Flags::Failed;
            };
            args_copy_value(&mut new, argument);
            *i += 1;
        }
        log_debug(
            c"%s: -%c = %s",
            fmt_args![
                c"args_parse_flag_argument".as_ptr(),
                flag as core::ffi::c_int,
                args_value_as_string(&new).as_ptr()
            ],
        );
        args_set(args, flag, Some(new), 0);
        Flags::More
    }
}

/// Reads the flags out of the word at `i`, which the command line has to spell
/// as one `-` followed by flag letters.
unsafe fn args_parse_flags(
    parse: &args_parse_t,
    values: &[args_value_t],
    i: &mut usize,
    args: &mut args,
    cause: &mut Option<CString>,
) -> Flags {
    unsafe {
        let value = &values[*i];
        let ArgsValue::String(string) = &value.value else {
            return Flags::Arguments;
        };
        log_debug(
            c"%s: next %s",
            fmt_args![c"args_parse_flags".as_ptr(), string.as_ptr()],
        );
        let word = string.as_bytes();
        let Some(flags) = word.strip_prefix(b"-") else {
            return Flags::Arguments;
        };
        if flags.is_empty() {
            return Flags::Arguments;
        }
        *i += 1;
        if flags == b"-" {
            return Flags::Arguments;
        }
        let template = parse.template.to_bytes();
        for (n, &flag) in flags.iter().enumerate() {
            if flag == b'?' {
                return Flags::Failed;
            }
            if !is_flag_byte(flag) {
                *cause = Some(xasprintf(
                    c"invalid flag -%c",
                    fmt_args![flag as core::ffi::c_int],
                ));
                return Flags::Failed;
            }
            let Some(argument) = flag_in_template(template, flag) else {
                *cause = Some(xasprintf(
                    c"unknown flag -%c",
                    fmt_args![flag as core::ffi::c_int],
                ));
                return Flags::Failed;
            };
            if argument == FlagArgument::None {
                log_debug(
                    c"%s: -%c",
                    fmt_args![c"args_parse_flags".as_ptr(), flag as core::ffi::c_int],
                );
                args_set(args, flag, None, 0);
                continue;
            }
            let rest = CStr::from_ptr(flags[n + 1..].as_ptr().cast::<core::ffi::c_char>());
            return args_parse_flag_argument(
                values,
                i,
                args,
                cause,
                rest,
                flag,
                argument == FlagArgument::Optional,
            );
        }
        Flags::More
    }
}

pub unsafe fn args_parse(
    parse: &args_parse_t,
    values: &[args_value_t],
    cause: &mut Option<CString>,
) -> Option<Box<args>> {
    unsafe {
        if values.is_empty() {
            return Some(args_create());
        }
        let mut args = args_create();
        let mut i: usize = 1;
        while i < values.len() {
            match args_parse_flags(parse, values, &mut i, &mut args, cause) {
                Flags::More => {}
                Flags::Arguments => break,
                Flags::Failed => return None,
            }
        }
        log_debug(
            c"%s: flags end at %zu of %zu",
            fmt_args![c"args_parse".as_ptr(), i, values.len()],
        );
        while i < values.len() {
            let value = &values[i];
            log_debug(
                c"%s: %zu = %s (type %s)",
                fmt_args![
                    c"args_parse".as_ptr(),
                    i,
                    args_value_as_string(value).as_ptr(),
                    args_value_type_to_string(&value.value)
                ],
            );
            let type_0 = match parse.cb {
                Some(cb) => {
                    let type_0 = cb(&args, args.count, cause);
                    if type_0 == ARGS_PARSE_INVALID {
                        return None;
                    }
                    type_0
                }
                None => ARGS_PARSE_STRING,
            };
            args.values.push(args_value_t::default());
            let new = args.values.last_mut().unwrap();
            args.count += 1;
            match type_0 {
                ARGS_PARSE_INVALID => fatalx(c"unexpected argument type", fmt_args![]),
                ARGS_PARSE_STRING => {
                    if !matches!(&value.value, ArgsValue::String(_)) {
                        *cause = Some(xasprintf(
                            c"argument %u must be \"string\"",
                            fmt_args![args.count],
                        ));
                        return None;
                    }
                    args_copy_value(new, value);
                }
                ARGS_PARSE_COMMANDS_OR_STRING => args_copy_value(new, value),
                ARGS_PARSE_COMMANDS => {
                    if !matches!(&value.value, ArgsValue::Commands { .. }) {
                        *cause = Some(xasprintf(
                            c"argument %u must be { commands }",
                            fmt_args![args.count],
                        ));
                        return None;
                    }
                    args_copy_value(new, value);
                }
                _ => {}
            }
            i += 1;
        }
        if parse.lower != -1 && args.count < parse.lower as u_int {
            *cause = Some(xasprintf(
                c"too few arguments (need at least %u)",
                fmt_args![parse.lower],
            ));
            return None;
        }
        if parse.upper != -1 && args.count > parse.upper as u_int {
            *cause = Some(xasprintf(
                c"too many arguments (need at most %u)",
                fmt_args![parse.upper],
            ));
            return None;
        }
        Some(args)
    }
}

/// Copies a value, replacing `%1` to `%9` in a string with the words given.
unsafe fn args_copy_copy_value(to: &mut args_value_t, from: &args_value_t, argv: &[CString]) {
    unsafe {
        to.value = match &from.value {
            ArgsValue::String(string) => {
                let mut expanded = string.clone();
                for (i, arg) in argv.iter().enumerate() {
                    expanded = cmd_template_replace(&expanded, arg, (i + 1) as core::ffi::c_int);
                }
                ArgsValue::String(expanded)
            }
            ArgsValue::Commands { cmdlist, .. } => ArgsValue::Commands {
                cmdlist: cmdlist.as_ref().map(|list| list.copy_with_arguments(argv)),
                cached: std::cell::OnceCell::new(),
            },
            ArgsValue::None => ArgsValue::None,
        };
    }
}

pub unsafe fn args_copy(args: &args, argv: &[CString]) -> Box<args> {
    unsafe {
        cmd_log_argv(argv, c"%s", fmt_args![c"args_copy".as_ptr()]);
        let mut new_args = args_create();
        for entry in args.tree.values() {
            if entry.values.is_empty() {
                for _ in 0..entry.count {
                    args_set(&mut new_args, entry.flag, None, 0);
                }
                continue;
            }
            for value in entry.values.iter() {
                let mut new_value = Box::new(args_value_t::default());
                args_copy_copy_value(&mut new_value, value, argv);
                args_set(&mut new_args, entry.flag, Some(new_value), 0);
            }
        }
        if args.count == 0 {
            return new_args;
        }
        new_args.count = args.count;
        new_args.values.reserve(args.values.len());
        for value in args.values.iter() {
            let mut new_value = args_value_t::default();
            args_copy_copy_value(&mut new_value, value, argv);
            new_args.values.push(new_value);
        }
        new_args
    }
}

pub unsafe fn args_to_vector(args: &args) -> Vec<CString> {
    unsafe {
        let mut argv = Vec::new();
        for value in args.values.iter() {
            match &value.value {
                ArgsValue::String(string) => argv.push(string.clone()),
                ArgsValue::Commands { cmdlist, .. } => {
                    let s = cmdlist
                        .as_ref()
                        .map_or_else(CString::default, |list| list.print(0));
                    argv.push(s);
                }
                ArgsValue::None => {}
            }
        }
        argv
    }
}

pub fn args_from_vector(argv: &[CString]) -> Vec<args_value_t> {
    argv.iter()
        .map(|arg| args_value_t {
            value: ArgsValue::String(arg.clone()),
        })
        .collect()
}

/// A C string of `text`, as the module hands its results back.
fn copy_of(text: &[u8]) -> CString {
    CString::new(text).expect("argument text cannot contain NUL")
}

/// Appends one value to the printed arguments, separated by a space from
/// whatever has been printed already.
unsafe fn args_print_add_value(out: &mut Vec<u8>, value: &args_value_t) {
    unsafe {
        if !out.is_empty() {
            out.push(b' ');
        }
        match &value.value {
            ArgsValue::Commands { cmdlist, .. } => {
                let expanded = cmdlist
                    .as_ref()
                    .map_or_else(CString::default, |list| list.print(0));
                out.extend_from_slice(b"{ ");
                out.extend_from_slice(expanded.as_bytes());
                out.extend_from_slice(b" }");
            }
            ArgsValue::String(string) => {
                let expanded = args_escape(string.as_c_str());
                out.extend_from_slice(expanded.as_bytes());
            }
            ArgsValue::None => {}
        }
    }
}

pub unsafe fn args_print(args: &args) -> CString {
    unsafe {
        let mut out: Vec<u8> = Vec::new();
        for entry in args.tree.values() {
            if entry.flags & ARGS_ENTRY_OPTIONAL_VALUE != 0 || !entry.values.is_empty() {
                continue;
            }
            if out.is_empty() {
                out.push(b'-');
            }
            for _ in 0..entry.count {
                out.push(entry.flag);
            }
        }
        let mut last: Option<&args_entry> = None;
        for entry in args.tree.values() {
            let flag = |out: &mut Vec<u8>| {
                if !out.is_empty() {
                    out.push(b' ');
                }
                out.push(b'-');
                out.push(entry.flag);
            };
            if entry.flags & ARGS_ENTRY_OPTIONAL_VALUE != 0 {
                flag(&mut out);
                last = Some(entry);
            } else if !entry.values.is_empty() {
                for value in entry.values.iter() {
                    flag(&mut out);
                    args_print_add_value(&mut out, value);
                }
                last = Some(entry);
            }
        }
        if last.is_some_and(|last| last.flags & ARGS_ENTRY_OPTIONAL_VALUE != 0) {
            out.extend_from_slice(b" --");
        }
        for value in args.values.iter() {
            args_print_add_value(&mut out, value);
        }
        copy_of(&out)
    }
}

/// The quoting a string needs to survive being read back as one word.
#[derive(Clone, Copy, PartialEq)]
enum Quotes {
    None,
    Single,
    Double,
}

/// The quoting a string needs: double quotes for the bytes the parser would
/// otherwise read as syntax, single quotes for the ones only they survive.
fn quotes_for(text: &[u8]) -> Quotes {
    if text.iter().any(|b| b" #';${}%".contains(b)) {
        Quotes::Double
    } else if text.iter().any(|b| b" \"".contains(b)) {
        Quotes::Single
    } else {
        Quotes::None
    }
}

pub(crate) fn args_escape_impl(s: &CStr) -> CString {
    unsafe {
        let text = s.to_bytes();
        let Some(&first) = text.first() else {
            return CString::from_vec_unchecked(b"''".to_vec());
        };
        let quotes = quotes_for(text);
        if first != b' ' && text.len() == 1 && (quotes != Quotes::None || first == b'~') {
            return CString::from_vec_unchecked(vec![b'\\', first]);
        }
        let mut flags = VIS_OCTAL | VIS_CSTYLE | VIS_TAB | VIS_NL;
        if quotes == Quotes::Double {
            flags |= VIS_DQ;
        }
        let escaped = RustUtf8VisModel.encode_utf8(text, flags);
        let visible = escaped.as_bytes();
        let tilde = visible.first() == Some(&b'~');
        let mut result: Vec<u8> = Vec::new();
        match quotes {
            Quotes::Single => {
                result.push(b'\'');
                result.extend_from_slice(visible);
                result.push(b'\'');
            }
            Quotes::Double => {
                result.push(b'"');
                if tilde {
                    result.push(b'\\');
                }
                result.extend_from_slice(visible);
                result.push(b'"');
            }
            Quotes::None => {
                if tilde {
                    result.push(b'\\');
                }
                result.extend_from_slice(visible);
            }
        }
        CString::from_vec_unchecked(result)
    }
}

pub fn args_has(args: &args, flag: u_char) -> core::ffi::c_int {
    match args_find(args, flag) {
        Some(entry) => entry.count as core::ffi::c_int,
        None => 0,
    }
}

pub unsafe fn args_set(
    args: &mut args,
    flag: u_char,
    value: Option<Box<args_value_t>>,
    flags: core::ffi::c_int,
) {
    let entry = args.tree.entry(flag).or_insert_with(|| {
        Box::new(args_entry {
            flag,
            values: Vec::new(),
            count: 0,
            flags,
        })
    });
    entry.count += 1;
    let Some(value) = value else {
        return;
    };
    if matches!(&value.value, ArgsValue::None) {
        return;
    }
    entry.values.push(value);
}

/// The last string value given for `flag`, borrowed from the arguments.
pub fn args_get_str(args: &args, flag: u_char) -> Option<&CStr> {
    match args_last_value(args, flag) {
        Some(value) => match &value.value {
            ArgsValue::String(string) => Some(string.as_c_str()),
            ArgsValue::None | ArgsValue::Commands { .. } => None,
        },
        None => None,
    }
}

/// The flags the arguments carry, in flag order. This is the walk the C's
/// `args_first` and `args_next` pair did through a cursor entry the caller
/// held for them.
pub fn args_flags(args: &args) -> impl Iterator<Item = u_char> + '_ {
    args.tree.values().map(|entry| entry.flag)
}

pub fn args_count(args: &args) -> u_int {
    args.count
}

pub fn args_value(args: &args, idx: u_int) -> Option<&args_value_t> {
    args.values.get(idx as usize)
}

/// Borrows the `idx`th argument as a string, or returns `None` when absent.
pub unsafe fn args_string_str(args: &args, idx: u_int) -> Option<&CStr> {
    unsafe {
        args.values
            .get(idx as usize)
            .map(|value| args_value_as_string(value))
    }
}

pub(crate) unsafe fn args_make_commands_now(
    self_0: &cmd,
    item: &cmdq_item,
    idx: u_int,
    expand: core::ffi::c_int,
) -> Option<CmdListRef> {
    unsafe {
        let mut state = args_make_commands_prepare(self_0, item, idx, None, 0, |cmd| {
            if expand != 0 {
                format_single_from_target(item, cmd)
            } else {
                cmd.to_owned()
            }
        });
        let mut error = None;
        let cmdlist = args_make_commands(&mut state, &[], &mut error);
        if let Some(error) = error.as_ref() {
            item.error(c"%s", fmt_args![error.as_ptr()]);
        }
        cmdlist
    }
}

/// Prepares a command list or owned command text for later argument substitution.
/// The expander runs once for text (including a default), and never for an
/// already parsed command list. Pass `CStr::to_owned` to preserve literal text.
pub fn args_make_commands_prepare(
    self_0: &cmd,
    item: &cmdq_item,
    idx: u_int,
    default_command: Option<&CStr>,
    wait: core::ffi::c_int,
    expand: impl FnOnce(&CStr) -> CString,
) -> Box<args_command_state> {
    let args = cmd_get_args(self_0);
    let target = &item.target;
    let tc = item.target_client();
    let mut state = Box::new(args_command_state {
        cmdlist: None,
        cmd: None,
        pi: cmd_parse_input::default(),
        source_file: None,
        client_ref: None,
    });
    let cmd = match args_value(args, idx) {
        Some(value) => {
            if let ArgsValue::Commands { cmdlist, .. } = &value.value {
                state.cmdlist = cmdlist.clone();
                return state;
            }
            let ArgsValue::String(string) = &value.value else {
                unsafe { fatalx(c"unexpected argument type", fmt_args![]) };
            };
            Some(string.as_c_str())
        }
        None => default_command,
    };
    let Some(cmd) = cmd else {
        unsafe { fatalx(c"argument out of range", fmt_args![]) };
    };
    state.cmd = Some(expand(cmd));
    unsafe {
        log_debug(
            c"%s: %s",
            fmt_args![c"args_make_commands_prepare", state.cmd.as_deref()],
        );
    }
    let (file, line) = cmd_get_source(self_0);
    state.pi.line = line;
    state.source_file = file.map(CStr::to_owned);
    state.pi.file = state.source_file.clone();
    state.pi.c = tc.as_ref().map(ClientRef::downgrade);
    state.client_ref = tc;
    cmd_find_copy_state(&mut state.pi.fs, target);
    if wait != 0 {
        state.pi.item = cmdq_item_ref_of(item).map(|item| item.downgrade());
    }
    state
}

pub(crate) unsafe fn args_make_commands(
    state: &mut args_command_state,
    argv: &[CString],
    error: &mut Option<CString>,
) -> Option<CmdListRef> {
    unsafe {
        if let Some(cmdlist) = state.cmdlist.as_ref() {
            if argv.is_empty() {
                return Some(cmdlist.clone());
            }
            return Some(cmdlist.copy_with_arguments(argv));
        }
        let mut cmd = match state.cmd.as_deref() {
            Some(c) => c.to_owned(),
            None => CString::default(),
        };
        log_debug(
            c"%s: %s",
            fmt_args![c"args_make_commands".as_ptr(), cmd.as_ptr()],
        );
        cmd_log_argv(argv, c"args_make_commands", fmt_args![]);
        for (i, arg) in argv.iter().enumerate() {
            let new_cmd = cmd_template_replace(&cmd, arg, (i + 1) as core::ffi::c_int);
            log_debug(
                c"%s: %%%u %s: %s",
                fmt_args![
                    c"args_make_commands".as_ptr(),
                    (i + 1) as core::ffi::c_uint,
                    arg.as_ptr(),
                    new_cmd.as_ptr()
                ],
            );
            cmd = new_cmd;
        }
        log_debug(
            c"%s: %s",
            fmt_args![c"args_make_commands".as_ptr(), cmd.as_ptr()],
        );
        let mut pr = cmd_parse_from_string(&cmd, Some(&mut state.pi));
        match pr.status {
            CMD_PARSE_ERROR => {
                *error = pr.error.take();
                None
            }
            CMD_PARSE_SUCCESS => pr.cmdlist.take(),
            _ => fatalx(c"invalid parse return state", fmt_args![]),
        }
    }
}

pub unsafe fn args_make_commands_get_command(state: &args_command_state) -> CString {
    unsafe {
        if let Some(cmdlist) = state.cmdlist.as_ref() {
            return match cmdlist.command(0) {
                Some(first) => crate::CommandEntry::name(cmd_get_entry(&first)).to_owned(),
                None => CString::default(),
            };
        }
        let cmd = state.cmd.as_deref().map(|c| c.to_bytes()).unwrap_or(b"");
        let end = cmd
            .iter()
            .position(|b| b" ,".contains(b))
            .unwrap_or(cmd.len());
        CString::from_vec_unchecked(cmd[..end].to_vec())
    }
}

/// Every value given for a flag, in the order they were given.
pub fn args_value_list(args: &args, flag: u_char) -> Vec<&args_value_t> {
    match args_find(args, flag) {
        Some(entry) => entry.values.iter().map(|value| &**value).collect(),
        None => Vec::new(),
    }
}

/// The number a string holds, or the `strtonum` message saying why it is not
/// one. The message is one of that module's own static strings.
fn number(
    s: &CStr,
    minval: core::ffi::c_longlong,
    maxval: core::ffi::c_longlong,
) -> Result<core::ffi::c_longlong, &'static CStr> {
    unsafe { strtonum(s, minval, maxval) }
}

/// Reports why an argument was not the number that was wanted.
fn no_number(cause: &mut Option<CString>, errstr: &CStr) -> core::ffi::c_longlong {
    *cause = Some(errstr.to_owned());
    0
}

pub fn args_strtonum(
    args: &args,
    flag: u_char,
    minval: core::ffi::c_longlong,
    maxval: core::ffi::c_longlong,
    cause: &mut Option<CString>,
) -> core::ffi::c_longlong {
    let Some(value) = args_last_string(args, flag) else {
        return no_number(cause, c"missing");
    };
    match number(value, minval, maxval) {
        Ok(ll) => {
            *cause = None;
            ll
        }
        Err(errstr) => no_number(cause, errstr),
    }
}

pub unsafe fn args_strtonum_and_expand(
    args: &args,
    flag: u_char,
    minval: core::ffi::c_longlong,
    maxval: core::ffi::c_longlong,
    item: &cmdq_item,
    cause: &mut Option<CString>,
) -> core::ffi::c_longlong {
    unsafe {
        let Some(value) = args_last_string(args, flag) else {
            return no_number(cause, c"missing");
        };
        let formatted = format_single_from_target(item, value);
        let result = number(&formatted, minval, maxval);
        match result {
            Ok(ll) => {
                *cause = None;
                ll
            }
            Err(errstr) => no_number(cause, errstr),
        }
    }
}

pub unsafe fn args_percentage(
    args: &args,
    flag: u_char,
    minval: core::ffi::c_longlong,
    maxval: core::ffi::c_longlong,
    curval: core::ffi::c_longlong,
    cause: &mut Option<CString>,
) -> core::ffi::c_longlong {
    let Some(entry) = args_find(args, flag) else {
        return no_number(cause, c"missing");
    };
    let Some(value) = entry.values.last() else {
        return no_number(cause, c"empty");
    };
    let ArgsValue::String(string) = &value.value else {
        return no_number(cause, c"missing");
    };
    args_string_percentage(string.as_c_str(), minval, maxval, curval, cause)
}

/// The number in front of the `%` of a percentage, when the value is one.
fn percentage_of(text: &[u8]) -> Option<&[u8]> {
    (text.last() == Some(&b'%')).then(|| &text[..text.len() - 1])
}

/// The share of `curval` a percentage stands for, checked against the range
/// the caller allows.
fn share_of(
    percent: core::ffi::c_longlong,
    minval: core::ffi::c_longlong,
    maxval: core::ffi::c_longlong,
    curval: core::ffi::c_longlong,
    cause: &mut Option<CString>,
) -> core::ffi::c_longlong {
    let ll = curval * percent / 100;
    if ll < minval {
        return no_number(cause, c"too small");
    }
    if ll > maxval {
        return no_number(cause, c"too large");
    }
    *cause = None;
    ll
}

pub(crate) fn args_string_percentage_impl(
    value: &CStr,
    minval: core::ffi::c_longlong,
    maxval: core::ffi::c_longlong,
    curval: core::ffi::c_longlong,
    cause: &mut Option<CString>,
) -> core::ffi::c_longlong {
    let text = value.to_bytes();
    if text.is_empty() {
        return no_number(cause, c"empty");
    }
    let Some(percent) = percentage_of(text) else {
        return match number(value, minval, maxval) {
            Ok(ll) => {
                *cause = None;
                ll
            }
            Err(errstr) => no_number(cause, errstr),
        };
    };
    let copy = copy_of(percent);
    let result = number(&copy, 0, 100);
    match result {
        Ok(percent) => share_of(percent, minval, maxval, curval, cause),
        Err(errstr) => no_number(cause, errstr),
    }
}

pub fn args_escape(s: &CStr) -> CString {
    crate::ArgumentTextCodec::escape(&crate::RustArgumentTextCodec, s)
}

pub fn args_string_percentage(
    value: &CStr,
    minval: core::ffi::c_longlong,
    maxval: core::ffi::c_longlong,
    curval: core::ffi::c_longlong,
    cause: &mut Option<CString>,
) -> core::ffi::c_longlong {
    crate::ArgumentTextCodec::percentage(
        &crate::RustArgumentTextCodec,
        value,
        minval,
        maxval,
        curval,
        cause,
    )
}

/// Like `args_string_percentage`, but expanding the value as a format first.
/// An empty value is read as a plain number here, which is what reading the
/// byte in front of the string used to come to.
pub unsafe fn args_string_percentage_and_expand(
    value: &CStr,
    minval: core::ffi::c_longlong,
    maxval: core::ffi::c_longlong,
    curval: core::ffi::c_longlong,
    item: &cmdq_item,
    cause: &mut Option<CString>,
) -> core::ffi::c_longlong {
    let text = value.to_bytes();
    let Some(percent) = percentage_of(text) else {
        let formatted = unsafe { format_single_from_target(item, value) };
        let result = number(&formatted, minval, maxval);
        return match result {
            Ok(ll) => {
                *cause = None;
                ll
            }
            Err(errstr) => no_number(cause, errstr),
        };
    };
    let copy = copy_of(percent);
    let formatted = unsafe { format_single_from_target(item, &copy) };
    let result = number(&formatted, 0, 100);
    match result {
        Ok(percent) => share_of(percent, minval, maxval, curval, cause),
        Err(errstr) => no_number(cause, errstr),
    }
}

pub unsafe fn args_percentage_and_expand(
    args: &args,
    flag: u_char,
    minval: core::ffi::c_longlong,
    maxval: core::ffi::c_longlong,
    curval: core::ffi::c_longlong,
    item: &cmdq_item,
    cause: &mut Option<CString>,
) -> core::ffi::c_longlong {
    unsafe {
        let Some(entry) = args_find(args, flag) else {
            return no_number(cause, c"missing");
        };
        let Some(value) = entry.values.last() else {
            return no_number(cause, c"empty");
        };
        let ArgsValue::String(string) = &value.value else {
            return no_number(cause, c"missing");
        };
        args_string_percentage_and_expand(string.as_c_str(), minval, maxval, curval, item, cause)
    }
}

#[cfg(test)]
#[path = "../tests/test_arguments.rs"]
mod tests;

#[cfg(test)]
pub(crate) use tests::args_free;
