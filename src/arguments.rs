use crate::cmd::cmd_find_copy_state;
use crate::cmd::cmd_parse_from_string;
use crate::cmd::{
    cmd_get_args_ptr, cmd_get_entry, cmd_get_source, cmd_list_copy, cmd_list_first, cmd_list_print,
};
use crate::cmd::{cmd_log_argv, cmd_template_replace};
use crate::cmd::{cmdq_error, cmdq_get_target, cmdq_get_target_client};
use crate::compat::strtonum;
use crate::ffi::__ctype_b_loc;
use crate::fmt_args;
use crate::format::format_single_from_target;
use crate::log::{fatalx, log_debug};
use crate::server::client_ref_from_ptr;
use crate::text::utf8_stravis;
pub use crate::types::*;
use crate::xmalloc::xasprintf;
use ::core::ops::Bound;
use ::std::ffi::CStr;
use ::std::ffi::CString;
pub type ctype_mask = ::core::ffi::c_uint;
pub const _ISalnum: ctype_mask = 8;
#[repr(C)]
pub struct args {
    pub tree: args_tree,
    pub count: u_int,
    pub values: Vec<args_value_t>,
}
#[repr(C)]
pub struct args_command_state {
    pub(crate) cmdlist: Option<CmdListRef>,
    pub cmd: Option<::std::ffi::CString>,
    pub pi: cmd_parse_input,
    pub(crate) source_file: Option<::std::ffi::CString>,
    pub(crate) client_ref: Option<ClientRef>,
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
pub const ARGS_PARSE_COMMANDS: args_parse_type = 3;
pub const ARGS_PARSE_COMMANDS_OR_STRING: args_parse_type = 2;
pub const ARGS_PARSE_STRING: args_parse_type = 1;
pub const ARGS_PARSE_INVALID: args_parse_type = 0;
pub const CMD_PARSE_SUCCESS: cmd_parse_status = 1;
pub const CMD_PARSE_ERROR: cmd_parse_status = 0;
pub const RB_BLACK: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const RB_RED: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const RB_NEGINF: ::core::ffi::c_int = -(1 as ::core::ffi::c_int);
pub const VIS_OCTAL: ::core::ffi::c_int = 0x1 as ::core::ffi::c_int;
pub const VIS_CSTYLE: ::core::ffi::c_int = 0x2 as ::core::ffi::c_int;
pub const VIS_TAB: ::core::ffi::c_int = 0x8 as ::core::ffi::c_int;
pub const VIS_NL: ::core::ffi::c_int = 0x10 as ::core::ffi::c_int;
pub const VIS_DQ: ::core::ffi::c_int = 0x200 as ::core::ffi::c_int;
pub const ARGS_ENTRY_OPTIONAL_VALUE: ::core::ffi::c_int = 0x1 as ::core::ffi::c_int;

/// A pointer that a walk stops at when it is null.
fn non_null<T>(p: *mut T) -> Option<*mut T> {
    (!p.is_null()).then_some(p)
}

/// The value at `i` of a command line, if the line has not run out.
unsafe fn value_at(values: *mut args_value_t, count: u_int, i: u_int) -> Option<*mut args_value_t> {
    (i < count).then(|| unsafe { values.add(i as usize) })
}

/// The entries of a flag tree, in flag order.
fn entries(tree: *mut args_tree) -> impl Iterator<Item = *mut args_entry> {
    let all: Vec<*mut args_entry> = unsafe {
        (*tree)
            .values()
            .map(|entry| entry.as_ref() as *const args_entry as *mut args_entry)
            .collect()
    };
    all.into_iter()
}

/// The values hanging off an entry, in the order they were given.
fn value_list(
    values: &[::std::boxed::Box<args_value_t>],
) -> impl Iterator<Item = *mut args_value_t> {
    values
        .iter()
        .map(|value| value.as_ref() as *const args_value_t as *mut args_value_t)
}

/// The entry a flag has in the arguments, if it has one.
unsafe fn args_find(args: &args, flag: u_char) -> *mut args_entry {
    args.tree
        .get(&flag)
        .map(|entry| entry.as_ref() as *const args_entry as *mut args_entry)
        .unwrap_or(::core::ptr::null_mut::<args_entry>())
}

/// The last value given for a flag, if it was given any.
unsafe fn args_last_value(args: &args, flag: u_char) -> Option<*mut args_value_t> {
    unsafe {
        let entry = non_null(args_find(args, flag))?;
        value_list(&(*entry).values).last()
    }
}

/// The string of the last value given for a flag, when there is one and it is
/// a string.
unsafe fn args_last_string(args: &args, flag: u_char) -> Option<*const ::core::ffi::c_char> {
    unsafe {
        let value = args_last_value(args, flag)?;
        let ArgsValue::String(string) = &(*value).value else {
            return None;
        };
        Some(string.as_ptr())
    }
}

/// Whether `b` is `isalnum` under the process's current locale, which is what
/// a flag has to be.
fn is_flag_byte(b: u8) -> bool {
    let class = unsafe { *(*__ctype_b_loc()).add(b as usize) } as ctype_mask;
    class & _ISalnum != 0
}

unsafe fn args_copy_value(to: *mut args_value_t, from: *const args_value_t) {
    unsafe {
        (*to).value = match &(*from).value {
            ArgsValue::Commands { cmdlist, .. } => ArgsValue::Commands {
                cmdlist: cmdlist.clone(),
                cached: None,
            },
            ArgsValue::String(string) => ArgsValue::String(string.clone()),
            ArgsValue::None => ArgsValue::None,
        };
    }
}

fn args_value_type_to_string(value: &ArgsValue) -> *const ::core::ffi::c_char {
    match value {
        ArgsValue::None => c"NONE".as_ptr(),
        ArgsValue::String(_) => c"STRING".as_ptr(),
        ArgsValue::Commands { .. } => c"COMMANDS".as_ptr(),
    }
}

unsafe fn args_value_as_string(value: &mut args_value_t) -> &CStr {
    unsafe {
        match &mut value.value {
            ArgsValue::None => c"",
            ArgsValue::Commands { cmdlist, cached } => {
                if cached.is_none() {
                    let printed = cmd_list_print(
                        cmdlist
                            .as_ref()
                            .map_or(::core::ptr::null(), |list| list.as_ptr()),
                        0,
                    );
                    *cached = Some(printed);
                }
                cached.as_ref().unwrap().as_c_str()
            }
            ArgsValue::String(string) => string.as_c_str(),
        }
    }
}

pub fn args_create() -> Box<args> {
    Box::new(args {
        tree: args_tree::new(),
        count: 0,
        values: Vec::new(),
    })
}

pub fn args_ptr(value: &Option<Box<args>>) -> *mut args {
    value
        .as_ref()
        .map(|args| &raw const **args as *mut args)
        .unwrap_or(::core::ptr::null_mut::<args>())
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
    values: *mut args_value_t,
    count: u_int,
    i: &mut u_int,
    args: *mut args,
    cause: &mut Option<CString>,
    rest: &::core::ffi::CStr,
    flag: u_char,
    optional: bool,
) -> Flags {
    unsafe {
        let mut new = Box::new(args_value_t::default());
        if !rest.is_empty() {
            new.value = ArgsValue::String(rest.to_owned());
        } else {
            let argument = value_at(values, count, *i);
            if argument.is_some_and(|argument| !matches!(&(*argument).value, ArgsValue::String(_)))
            {
                *cause = Some(xasprintf(
                    c"-%c argument must be a string".as_ptr(),
                    fmt_args![flag as ::core::ffi::c_int],
                ));
                drop(new);
                return Flags::Failed;
            }
            let Some(argument) = argument else {
                drop(new);
                if optional {
                    log_debug(
                        c"%s: -%c (optional)".as_ptr(),
                        fmt_args![
                            c"args_parse_flag_argument".as_ptr(),
                            flag as ::core::ffi::c_int
                        ],
                    );
                    args_set(args, flag, None, ARGS_ENTRY_OPTIONAL_VALUE);
                    return Flags::More;
                }
                *cause = Some(xasprintf(
                    c"-%c expects an argument".as_ptr(),
                    fmt_args![flag as ::core::ffi::c_int],
                ));
                return Flags::Failed;
            };
            args_copy_value(&raw mut *new, argument);
            *i += 1;
        }
        log_debug(
            c"%s: -%c = %s".as_ptr(),
            fmt_args![
                c"args_parse_flag_argument".as_ptr(),
                flag as ::core::ffi::c_int,
                args_value_as_string(&mut new).as_ptr()
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
    values: *mut args_value_t,
    count: u_int,
    i: &mut u_int,
    args: *mut args,
    cause: &mut Option<CString>,
) -> Flags {
    unsafe {
        let value = values.add(*i as usize);
        let ArgsValue::String(string) = &(*value).value else {
            return Flags::Arguments;
        };
        log_debug(
            c"%s: next %s".as_ptr(),
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
                    c"invalid flag -%c".as_ptr(),
                    fmt_args![flag as ::core::ffi::c_int],
                ));
                return Flags::Failed;
            }
            let Some(argument) = flag_in_template(template, flag) else {
                *cause = Some(xasprintf(
                    c"unknown flag -%c".as_ptr(),
                    fmt_args![flag as ::core::ffi::c_int],
                ));
                return Flags::Failed;
            };
            if argument == FlagArgument::None {
                log_debug(
                    c"%s: -%c".as_ptr(),
                    fmt_args![c"args_parse_flags".as_ptr(), flag as ::core::ffi::c_int],
                );
                args_set(args, flag, None, 0);
                continue;
            }
            let rest =
                ::core::ffi::CStr::from_ptr(flags[n + 1..].as_ptr().cast::<::core::ffi::c_char>());
            return args_parse_flag_argument(
                values,
                count,
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
    parse: *const args_parse_t,
    values: *mut args_value_t,
    count: u_int,
    cause: &mut Option<CString>,
) -> Option<Box<args>> {
    unsafe {
        if count == 0 {
            return Some(args_create());
        }
        let parse = &*parse;
        let mut args = args_create();
        let args_ptr = &raw mut *args;
        let mut i: u_int = 1;
        while i < count {
            match args_parse_flags(parse, values, count, &mut i, args_ptr, cause) {
                Flags::More => {}
                Flags::Arguments => break,
                Flags::Failed => return None,
            }
        }
        log_debug(
            c"%s: flags end at %u of %u".as_ptr(),
            fmt_args![c"args_parse".as_ptr(), i, count],
        );
        while i < count {
            let value = values.add(i as usize);
            log_debug(
                c"%s: %u = %s (type %s)".as_ptr(),
                fmt_args![
                    c"args_parse".as_ptr(),
                    i,
                    args_value_as_string(&mut *value).as_ptr(),
                    args_value_type_to_string(&(*value).value)
                ],
            );
            let type_0 = match parse.cb {
                Some(cb) => {
                    let type_0 = cb(&*args_ptr, (*args_ptr).count, cause);
                    if type_0 == ARGS_PARSE_INVALID {
                        return None;
                    }
                    type_0
                }
                None => ARGS_PARSE_STRING,
            };
            (*args_ptr).values.push(args_value_t::default());
            let new = (*args_ptr).values.last_mut().unwrap() as *mut args_value_t;
            (*args_ptr).count += 1;
            match type_0 {
                ARGS_PARSE_INVALID => fatalx(c"unexpected argument type".as_ptr(), fmt_args![]),
                ARGS_PARSE_STRING => {
                    if !matches!(&(*value).value, ArgsValue::String(_)) {
                        *cause = Some(xasprintf(
                            c"argument %u must be \"string\"".as_ptr(),
                            fmt_args![(*args_ptr).count],
                        ));
                        return None;
                    }
                    args_copy_value(new, value);
                }
                ARGS_PARSE_COMMANDS_OR_STRING => args_copy_value(new, value),
                ARGS_PARSE_COMMANDS => {
                    if !matches!(&(*value).value, ArgsValue::Commands { .. }) {
                        *cause = Some(xasprintf(
                            c"argument %u must be { commands }".as_ptr(),
                            fmt_args![(*args_ptr).count],
                        ));
                        return None;
                    }
                    args_copy_value(new, value);
                }
                _ => {}
            }
            i += 1;
        }
        if parse.lower != -1 && (*args_ptr).count < parse.lower as u_int {
            *cause = Some(xasprintf(
                c"too few arguments (need at least %u)".as_ptr(),
                fmt_args![parse.lower],
            ));
            return None;
        }
        if parse.upper != -1 && (*args_ptr).count > parse.upper as u_int {
            *cause = Some(xasprintf(
                c"too many arguments (need at most %u)".as_ptr(),
                fmt_args![parse.upper],
            ));
            return None;
        }
        Some(args)
    }
}

/// Copies a value, replacing `%1` to `%9` in a string with the words given.
unsafe fn args_copy_copy_value(to: *mut args_value_t, from: *const args_value_t, argv: &[CString]) {
    unsafe {
        (*to).value = match &(*from).value {
            ArgsValue::String(string) => {
                let mut expanded = string.clone();
                for (i, arg) in argv.iter().enumerate() {
                    expanded = cmd_template_replace(
                        expanded.as_ptr(),
                        arg.as_ptr(),
                        (i + 1) as ::core::ffi::c_int,
                    );
                }
                ArgsValue::String(expanded)
            }
            ArgsValue::Commands { cmdlist, .. } => ArgsValue::Commands {
                cmdlist: cmdlist
                    .as_ref()
                    .map(|list| cmd_list_copy(list.as_ptr(), argv)),
                cached: None,
            },
            ArgsValue::None => ArgsValue::None,
        };
    }
}

pub unsafe fn args_copy(args: *mut args, argv: &[CString]) -> Box<args> {
    unsafe {
        cmd_log_argv(argv, c"%s".as_ptr(), fmt_args![c"args_copy".as_ptr()]);
        let mut new_args = args_create();
        let new_args_ptr = &raw mut *new_args;
        for entry in entries(&raw mut (*args).tree) {
            if (*entry).values.is_empty() {
                for _ in 0..(*entry).count {
                    args_set(new_args_ptr, (*entry).flag, None, 0);
                }
                continue;
            }
            for value in value_list(&(*entry).values) {
                let mut new_value = Box::new(args_value_t::default());
                args_copy_copy_value(&raw mut *new_value, value, argv);
                args_set(new_args_ptr, (*entry).flag, Some(new_value), 0);
            }
        }
        if (*args).count == 0 {
            return new_args;
        }
        (*new_args_ptr).count = (*args).count;
        (*new_args_ptr).values.reserve((*args).values.len());
        for value in (*args).values.iter() {
            let mut new_value = args_value_t::default();
            args_copy_copy_value(&raw mut new_value, value as *const args_value_t, argv);
            (*new_args_ptr).values.push(new_value);
        }
        new_args
    }
}

pub unsafe fn args_free_value(value: *mut args_value_t) {
    unsafe {
        match ::core::mem::replace(&mut (*value).value, ArgsValue::None) {
            ArgsValue::String(string) => drop(string),
            ArgsValue::Commands { cmdlist, cached } => drop((cmdlist, cached)),
            ArgsValue::None => {}
        }
    }
}

pub unsafe fn args_free_values(values: *mut args_value_t, count: u_int) {
    unsafe {
        for i in 0..count as usize {
            args_free_value(values.add(i));
        }
    }
}

pub fn args_free(args: Box<args>) {
    drop(args);
}

pub unsafe fn args_to_vector(args: &args) -> Vec<CString> {
    unsafe {
        let mut argv = Vec::new();
        for value in args.values.iter() {
            match &value.value {
                ArgsValue::String(string) => argv.push(string.clone()),
                ArgsValue::Commands { cmdlist, .. } => {
                    let s = cmd_list_print(
                        cmdlist
                            .as_ref()
                            .map_or(::core::ptr::null(), |list| list.as_ptr()),
                        0,
                    );
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
            ..args_value_t::default()
        })
        .collect()
}

/// A C string of `text`, as the module hands its results back.
fn copy_of(text: &[u8]) -> CString {
    CString::new(text).expect("argument text cannot contain NUL")
}

/// Appends one value to the printed arguments, separated by a space from
/// whatever has been printed already.
unsafe fn args_print_add_value(out: &mut Vec<u8>, value: *mut args_value_t) {
    unsafe {
        if !out.is_empty() {
            out.push(b' ');
        }
        match &(*value).value {
            ArgsValue::Commands { cmdlist, .. } => {
                let expanded = cmd_list_print(
                    cmdlist
                        .as_ref()
                        .map_or(::core::ptr::null(), |list| list.as_ptr()),
                    0,
                );
                out.extend_from_slice(b"{ ");
                out.extend_from_slice(expanded.as_bytes());
                out.extend_from_slice(b" }");
            }
            ArgsValue::String(string) => {
                let expanded = args_escape(string.as_ptr());
                out.extend_from_slice(expanded.as_bytes());
            }
            ArgsValue::None => {}
        }
    }
}

pub unsafe fn args_print(args: *mut args) -> CString {
    unsafe {
        let mut out: Vec<u8> = Vec::new();
        for entry in entries(&raw mut (*args).tree) {
            if (*entry).flags & ARGS_ENTRY_OPTIONAL_VALUE != 0 || !(*entry).values.is_empty() {
                continue;
            }
            if out.is_empty() {
                out.push(b'-');
            }
            for _ in 0..(*entry).count {
                out.push((*entry).flag);
            }
        }
        let mut last: *mut args_entry = ::core::ptr::null_mut::<args_entry>();
        for entry in entries(&raw mut (*args).tree) {
            let flag = |out: &mut Vec<u8>| {
                if !out.is_empty() {
                    out.push(b' ');
                }
                out.push(b'-');
                out.push((*entry).flag);
            };
            if (*entry).flags & ARGS_ENTRY_OPTIONAL_VALUE != 0 {
                flag(&mut out);
                last = entry;
            } else if !(*entry).values.is_empty() {
                for value in value_list(&(*entry).values) {
                    flag(&mut out);
                    args_print_add_value(&mut out, value);
                }
                last = entry;
            }
        }
        if !last.is_null() && (*last).flags & ARGS_ENTRY_OPTIONAL_VALUE != 0 {
            out.extend_from_slice(b" --");
        }
        for value in (*args).values.iter_mut() {
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

pub unsafe fn args_escape(s: *const ::core::ffi::c_char) -> CString {
    unsafe {
        let text = ::core::ffi::CStr::from_ptr(s).to_bytes();
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
        let escaped = utf8_stravis(s, flags);
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

pub unsafe fn args_has(args: &args, flag: u_char) -> ::core::ffi::c_int {
    unsafe {
        match non_null(args_find(args, flag)) {
            Some(entry) => (*entry).count as ::core::ffi::c_int,
            None => 0,
        }
    }
}

pub unsafe fn args_set(
    args: *mut args,
    flag: u_char,
    value: Option<Box<args_value_t>>,
    flags: ::core::ffi::c_int,
) {
    unsafe {
        let entry = match non_null(args_find(&*args, flag)) {
            Some(entry) => {
                (*entry).count += 1;
                entry
            }
            None => {
                let mut entry = Box::new(args_entry {
                    flag,
                    values: Vec::new(),
                    count: 1,
                    flags,
                });
                let entry_ptr = entry.as_mut() as *mut args_entry;
                (*args).tree.insert(flag, entry);
                entry_ptr
            }
        };
        let Some(mut value) = value else {
            return;
        };
        if matches!(&value.value, ArgsValue::None) {
            return;
        }
        (*entry).values.push(value);
    }
}

/// The last string value given for `flag`, borrowed from the arguments.
pub unsafe fn args_get_str(args: &args, flag: u_char) -> Option<&::core::ffi::CStr> {
    unsafe {
        match args_last_value(args, flag) {
            Some(value) => match &(*value).value {
                ArgsValue::String(string) => Some(string.as_c_str()),
                ArgsValue::None | ArgsValue::Commands { .. } => None,
            },
            None => None,
        }
    }
}

pub unsafe fn args_get(args: &args, flag: u_char) -> *const ::core::ffi::c_char {
    unsafe {
        match args_last_value(args, flag) {
            Some(value) => match &(*value).value {
                ArgsValue::String(string) => string.as_ptr(),
                ArgsValue::None | ArgsValue::Commands { .. } => {
                    ::core::ptr::null::<::core::ffi::c_char>()
                }
            },
            None => ::core::ptr::null::<::core::ffi::c_char>(),
        }
    }
}

pub unsafe fn args_first(args: *mut args, entry: *mut *mut args_entry) -> u_char {
    unsafe {
        *entry = (*args)
            .tree
            .values()
            .next()
            .map(|entry| entry.as_ref() as *const args_entry as *mut args_entry)
            .unwrap_or(::core::ptr::null_mut::<args_entry>());
        match non_null(*entry) {
            Some(entry) => (*entry).flag,
            None => 0,
        }
    }
}

pub unsafe fn args_next(args: *mut args, entry: *mut *mut args_entry) -> u_char {
    unsafe {
        *entry = (*args)
            .tree
            .range((Bound::Excluded((**entry).flag), Bound::Unbounded))
            .next()
            .map(|(_, entry)| entry.as_ref() as *const args_entry as *mut args_entry)
            .unwrap_or(::core::ptr::null_mut::<args_entry>());
        match non_null(*entry) {
            Some(entry) => (*entry).flag,
            None => 0,
        }
    }
}

pub fn args_count(args: &args) -> u_int {
    args.count
}

pub unsafe fn args_values(args: *mut args) -> *mut args_value_t {
    unsafe {
        if (*args).values.is_empty() {
            ::core::ptr::null_mut::<args_value_t>()
        } else {
            (*args).values.as_mut_ptr()
        }
    }
}

pub unsafe fn args_value(args: *mut args, idx: u_int) -> *mut args_value_t {
    unsafe {
        (*args)
            .values
            .get_mut(idx as usize)
            .map(|value| value as *mut args_value_t)
            .unwrap_or(::core::ptr::null_mut::<args_value_t>())
    }
}

pub unsafe fn args_string(args: &args, idx: u_int) -> *const ::core::ffi::c_char {
    unsafe {
        match non_null(args_value(&raw const *args as *mut args, idx)) {
            Some(value) => args_value_as_string(&mut *value).as_ptr(),
            None => ::core::ptr::null::<::core::ffi::c_char>(),
        }
    }
}

pub(crate) unsafe fn args_make_commands_now(
    self_0: &cmd,
    item: *mut cmdq_item,
    idx: u_int,
    expand: ::core::ffi::c_int,
) -> Option<CmdListRef> {
    unsafe {
        let mut state = args_make_commands_prepare(
            self_0,
            item,
            idx,
            ::core::ptr::null::<::core::ffi::c_char>(),
            0,
            expand,
        );
        let mut error = None;
        let cmdlist = args_make_commands(&mut state, &[], &mut error);
        if let Some(error) = error.as_ref() {
            cmdq_error(item, c"%s".as_ptr(), fmt_args![error.as_ptr()]);
        }
        cmdlist
    }
}

pub unsafe fn args_make_commands_prepare(
    self_0: &cmd,
    item: *mut cmdq_item,
    idx: u_int,
    default_command: *const ::core::ffi::c_char,
    wait: ::core::ffi::c_int,
    expand: ::core::ffi::c_int,
) -> Box<args_command_state> {
    unsafe {
        let args = cmd_get_args_ptr(self_0);
        let target = cmdq_get_target(item);
        let tc = cmdq_get_target_client(&*item);
        let mut state = Box::new(args_command_state {
            cmdlist: None,
            cmd: None,
            pi: cmd_parse_input::default(),
            source_file: None,
            client_ref: None,
        });
        let cmd = match non_null(args_value(args, idx)) {
            Some(value) => {
                if let ArgsValue::Commands { cmdlist, .. } = &(*value).value {
                    state.cmdlist = cmdlist.clone();
                    return state;
                }
                let ArgsValue::String(string) = &(*value).value else {
                    fatalx(c"unexpected argument type".as_ptr(), fmt_args![]);
                };
                string.as_ptr()
            }
            None => {
                if default_command.is_null() {
                    fatalx(c"argument out of range".as_ptr(), fmt_args![]);
                }
                default_command
            }
        };
        state.cmd = if expand != 0 {
            Some(format_single_from_target(item, CStr::from_ptr(cmd)))
        } else {
            Some(CStr::from_ptr(cmd).to_owned())
        };
        log_debug(
            c"%s: %s".as_ptr(),
            fmt_args![c"args_make_commands_prepare".as_ptr(), cstr_ptr(&state.cmd)],
        );
        if wait != 0 {
            state.pi.item = item;
        }
        let file: *const ::core::ffi::c_char;
        (file, state.pi.line) = cmd_get_source(self_0);
        if !file.is_null() {
            state.source_file = Some(CStr::from_ptr(file).to_owned());
            state.pi.file = state.source_file.clone();
        }
        state.pi.c = client_ref_from_ptr(tc).map(|c| c.downgrade());
        state.client_ref = client_ref_from_ptr(tc);
        cmd_find_copy_state(&mut state.pi.fs, &*target);
        state
    }
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
            return Some(cmd_list_copy(cmdlist.as_ptr(), argv));
        }
        let mut cmd = match state.cmd.as_deref() {
            Some(c) => c.to_owned(),
            None => CString::default(),
        };
        log_debug(
            c"%s: %s".as_ptr(),
            fmt_args![c"args_make_commands".as_ptr(), cmd.as_ptr()],
        );
        cmd_log_argv(argv, c"args_make_commands".as_ptr(), fmt_args![]);
        for (i, arg) in argv.iter().enumerate() {
            let new_cmd =
                cmd_template_replace(cmd.as_ptr(), arg.as_ptr(), (i + 1) as ::core::ffi::c_int);
            log_debug(
                c"%s: %%%u %s: %s".as_ptr(),
                fmt_args![
                    c"args_make_commands".as_ptr(),
                    (i + 1) as ::core::ffi::c_uint,
                    arg.as_ptr(),
                    new_cmd.as_ptr()
                ],
            );
            cmd = new_cmd;
        }
        log_debug(
            c"%s: %s".as_ptr(),
            fmt_args![c"args_make_commands".as_ptr(), cmd.as_ptr()],
        );
        let mut pr = cmd_parse_from_string(cmd.as_ptr(), &raw mut state.pi);
        match pr.status {
            CMD_PARSE_ERROR => {
                *error = pr.error.take();
                None
            }
            CMD_PARSE_SUCCESS => pr.cmdlist.take(),
            _ => fatalx(c"invalid parse return state".as_ptr(), fmt_args![]),
        }
    }
}

pub unsafe fn args_make_commands_get_command(state: &args_command_state) -> CString {
    unsafe {
        if let Some(cmdlist) = state.cmdlist.as_ref() {
            return match non_null(cmd_list_first(cmdlist.as_ptr())) {
                Some(first) => cmd_get_entry(&*first).name.to_owned(),
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
pub unsafe fn args_value_list(args: &args, flag: u_char) -> Vec<*mut args_value_t> {
    unsafe {
        match non_null(args_find(args, flag)) {
            Some(entry) => value_list(&(*entry).values).collect(),
            None => Vec::new(),
        }
    }
}

/// The number a string holds, or the `strtonum` message saying why it is not
/// one. The message is one of that module's own static strings.
unsafe fn number(
    s: *const ::core::ffi::c_char,
    minval: ::core::ffi::c_longlong,
    maxval: ::core::ffi::c_longlong,
) -> Result<::core::ffi::c_longlong, *const ::core::ffi::c_char> {
    unsafe { strtonum(s, minval, maxval).map_err(::core::ffi::CStr::as_ptr) }
}

/// Reports why an argument was not the number that was wanted.
unsafe fn no_number(
    cause: &mut Option<CString>,
    errstr: *const ::core::ffi::c_char,
) -> ::core::ffi::c_longlong {
    unsafe {
        *cause = Some(CStr::from_ptr(errstr).to_owned());
        0
    }
}

pub unsafe fn args_strtonum(
    args: &args,
    flag: u_char,
    minval: ::core::ffi::c_longlong,
    maxval: ::core::ffi::c_longlong,
    cause: &mut Option<CString>,
) -> ::core::ffi::c_longlong {
    unsafe {
        let Some(value) = args_last_string(args, flag) else {
            return no_number(cause, c"missing".as_ptr());
        };
        match number(value, minval, maxval) {
            Ok(ll) => {
                *cause = None;
                ll
            }
            Err(errstr) => no_number(cause, errstr),
        }
    }
}

pub unsafe fn args_strtonum_and_expand(
    args: &args,
    flag: u_char,
    minval: ::core::ffi::c_longlong,
    maxval: ::core::ffi::c_longlong,
    item: *mut cmdq_item,
    cause: &mut Option<CString>,
) -> ::core::ffi::c_longlong {
    unsafe {
        let Some(value) = args_last_string(args, flag) else {
            return no_number(cause, c"missing".as_ptr());
        };
        let formatted = format_single_from_target(item, CStr::from_ptr(value));
        let result = number(formatted.as_ptr(), minval, maxval);
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
    minval: ::core::ffi::c_longlong,
    maxval: ::core::ffi::c_longlong,
    curval: ::core::ffi::c_longlong,
    cause: &mut Option<CString>,
) -> ::core::ffi::c_longlong {
    unsafe {
        let Some(entry) = non_null(args_find(args, flag)) else {
            return no_number(cause, c"missing".as_ptr());
        };
        let Some(value) = value_list(&(*entry).values).last() else {
            return no_number(cause, c"empty".as_ptr());
        };
        let ArgsValue::String(string) = &(*value).value else {
            return no_number(cause, c"missing".as_ptr());
        };
        args_string_percentage(string.as_ptr(), minval, maxval, curval, cause)
    }
}

/// The number in front of the `%` of a percentage, when the value is one.
fn percentage_of(text: &[u8]) -> Option<&[u8]> {
    (text.last() == Some(&b'%')).then(|| &text[..text.len() - 1])
}

/// The share of `curval` a percentage stands for, checked against the range
/// the caller allows.
unsafe fn share_of(
    percent: ::core::ffi::c_longlong,
    minval: ::core::ffi::c_longlong,
    maxval: ::core::ffi::c_longlong,
    curval: ::core::ffi::c_longlong,
    cause: &mut Option<CString>,
) -> ::core::ffi::c_longlong {
    unsafe {
        let ll = curval * percent / 100;
        if ll < minval {
            return no_number(cause, c"too small".as_ptr());
        }
        if ll > maxval {
            return no_number(cause, c"too large".as_ptr());
        }
        *cause = None;
        ll
    }
}

pub unsafe fn args_string_percentage(
    value: *const ::core::ffi::c_char,
    minval: ::core::ffi::c_longlong,
    maxval: ::core::ffi::c_longlong,
    curval: ::core::ffi::c_longlong,
    cause: &mut Option<CString>,
) -> ::core::ffi::c_longlong {
    unsafe {
        let text = ::core::ffi::CStr::from_ptr(value).to_bytes();
        if text.is_empty() {
            return no_number(cause, c"empty".as_ptr());
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
        let result = number(copy.as_ptr(), 0, 100);
        match result {
            Ok(percent) => share_of(percent, minval, maxval, curval, cause),
            Err(errstr) => no_number(cause, errstr),
        }
    }
}

/// Like `args_string_percentage`, but expanding the value as a format first.
/// An empty value is read as a plain number here, which is what reading the
/// byte in front of the string used to come to.
pub unsafe fn args_string_percentage_and_expand(
    value: *const ::core::ffi::c_char,
    minval: ::core::ffi::c_longlong,
    maxval: ::core::ffi::c_longlong,
    curval: ::core::ffi::c_longlong,
    item: *mut cmdq_item,
    cause: &mut Option<CString>,
) -> ::core::ffi::c_longlong {
    unsafe {
        let text = ::core::ffi::CStr::from_ptr(value).to_bytes();
        let Some(percent) = percentage_of(text) else {
            let formatted = format_single_from_target(item, CStr::from_ptr(value));
            let result = number(formatted.as_ptr(), minval, maxval);
            return match result {
                Ok(ll) => {
                    *cause = None;
                    ll
                }
                Err(errstr) => no_number(cause, errstr),
            };
        };
        let copy = copy_of(percent);
        let formatted = format_single_from_target(item, &copy);
        let result = number(formatted.as_ptr(), 0, 100);
        match result {
            Ok(percent) => share_of(percent, minval, maxval, curval, cause),
            Err(errstr) => no_number(cause, errstr),
        }
    }
}

pub unsafe fn args_percentage_and_expand(
    args: &args,
    flag: u_char,
    minval: ::core::ffi::c_longlong,
    maxval: ::core::ffi::c_longlong,
    curval: ::core::ffi::c_longlong,
    item: *mut cmdq_item,
    cause: &mut Option<CString>,
) -> ::core::ffi::c_longlong {
    unsafe {
        let Some(entry) = non_null(args_find(args, flag)) else {
            return no_number(cause, c"missing".as_ptr());
        };
        let Some(value) = value_list(&(*entry).values).last() else {
            return no_number(cause, c"empty".as_ptr());
        };
        let ArgsValue::String(string) = &(*value).value else {
            return no_number(cause, c"missing".as_ptr());
        };
        args_string_percentage_and_expand(string.as_ptr(), minval, maxval, curval, item, cause)
    }
}

#[cfg(test)]
#[path = "tests/test_arguments.rs"]
mod tests;
