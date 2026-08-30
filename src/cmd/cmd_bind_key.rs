//! `bind-key`: binds a key in a key table to a command list.
//!
//! What the exec hook decides is which table the binding goes into — `-T`
//! names one, `-n` means the root table, and otherwise it is the prefix table
//! — whether the binding repeats, what note it carries, and where its command
//! list comes from. A key on its own binds no list at all, which leaves
//! whatever the key was bound to in place; a brace-enclosed list the arguments
//! already hold is shared as it stands, with a reference taken once the
//! binding is in place; and anything else is handed back to the command
//! parser, whose list the binding takes over.
//!
//! The tables and the bindings in them belong to `key_bindings`, which is what
//! creates a table that is not there yet and what owns every binding in it;
//! nothing here reaches into those trees.
//!
//! Coverage exemptions: none.
use crate::arguments::{args_count, args_get, args_has, args_string, args_value, args_values};
use crate::cmd::parse::{cmd_parse_from_arguments, cmd_parse_from_string};
use crate::cmd::queue::cmdq_error;
use crate::cmd::{cmd_get_args, cmd_get_args_ptr};
use crate::fmt_args;
use crate::key_bindings::key_bindings_add;
use crate::text::key_string_lookup_string;
pub use crate::types::*;
use ::core::ffi::CStr;
use ::core::ptr::null_mut;
use ::std::ffi::CString;
pub type keyc = ::core::ffi::c_ulong;
pub const KEYC_F5: keyc = 8589934604;
pub const KEYC_UNKNOWN: keyc = 8589934593;
pub const KEYC_NONE: keyc = 8589934592;
pub const ARGS_PARSE_COMMANDS_OR_STRING: args_parse_type = 2;
pub const CMD_FIND_PANE: cmd_find_type = 0;
pub const CMD_RETURN_NORMAL: cmd_retval = 0;
pub const CMD_RETURN_ERROR: cmd_retval = -1;
pub const CMD_PARSE_ERROR: cmd_parse_status = 0;
pub const CMD_AFTERHOOK: ::core::ffi::c_int = 0x4 as ::core::ffi::c_int;
pub(crate) static cmd_bind_key_entry: cmd_entry = cmd_entry {
    name: c"bind-key",
    alias: Some(c"bind"),
    args: args_parse_t {
        template: c"nrN:T:",
        lower: 1,
        upper: -1,
        cb: Some(cmd_bind_key_args_parse),
    },
    usage: c"[-nr] [-T key-table] [-N note] key [command [argument ...]]",
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
    flags: CMD_AFTERHOOK,
    exec: cmd_bind_key_exec,
};

/// Where the command list a `bind-key` line binds comes from.
enum Binding {
    /// The line names no command, so no list is bound and the key keeps
    /// whatever it was bound to.
    Keep,
    /// A command list the arguments already hold, which the binding shares.
    Shared(CmdListRef),
    /// A command list the parser built from the words after the key, whose
    /// only reference the binding takes over.
    Owned(CmdListRef),
}

impl Binding {
    /// The list to bind, which is nothing at all when the line named no
    /// command.
    fn into_cmdlist(self) -> Option<CmdListRef> {
        match self {
            Binding::Keep => None,
            Binding::Shared(cmdlist) | Binding::Owned(cmdlist) => Some(cmdlist),
        }
    }
}

/// The key table the binding goes into.
unsafe fn table_name<'a>(args: &args) -> &'a CStr {
    unsafe {
        if args_has(args, b'T') != 0 {
            CStr::from_ptr(args_get(args, b'T'))
        } else if args_has(args, b'n') != 0 {
            c"root"
        } else {
            c"prefix"
        }
    }
}

/// Reads what the line binds to the key, answering the parser's error message
/// — which the caller gives back — when the words after the key are not a
/// command.
unsafe fn binding_of(args: *mut args, count: u_int) -> Result<Binding, CString> {
    unsafe {
        if count == 1 {
            return Ok(Binding::Keep);
        }
        let value = args_value(args, 1);
        if count == 2
            && let ArgsValue::Commands { cmdlist, .. } = &(*value).value
        {
            return Ok(Binding::Shared(cmdlist.clone().unwrap()));
        }
        let mut pr = if count == 2 {
            cmd_parse_from_string(args_string(&*args, 1), null_mut::<cmd_parse_input>())
        } else {
            cmd_parse_from_arguments(
                args_values(args).add(1),
                count.wrapping_sub(1),
                null_mut::<cmd_parse_input>(),
            )
        };
        if pr.status == CMD_PARSE_ERROR {
            return Err(pr.error.take().unwrap());
        }
        Ok(Binding::Owned(pr.cmdlist.take().unwrap()))
    }
}

unsafe fn cmd_bind_key_args_parse(
    _args: &args,
    _idx: u_int,
    _cause: &mut Option<CString>,
) -> args_parse_type {
    ARGS_PARSE_COMMANDS_OR_STRING
}

unsafe fn cmd_bind_key_exec(self_0: &cmd, item: *mut cmdq_item) -> cmd_retval {
    unsafe {
        let args = cmd_get_args(self_0);
        let note = args_get(args, b'N');
        let count = args_count(args);

        let keyname = args_string(args, 0);
        let key = key_string_lookup_string(keyname);
        if key == KEYC_NONE || key == KEYC_UNKNOWN {
            cmdq_error(item, c"unknown key: %s".as_ptr(), fmt_args![keyname]);
            return CMD_RETURN_ERROR;
        }

        let tablename = table_name(args);
        let repeat = args_has(args, b'r');

        let binding = match binding_of(cmd_get_args_ptr(self_0), count) {
            Ok(binding) => binding,
            Err(error) => {
                cmdq_error(item, c"%s".as_ptr(), fmt_args![error.as_ptr()]);
                return CMD_RETURN_ERROR;
            }
        };
        key_bindings_add(
            tablename.as_ptr(),
            key,
            note,
            repeat,
            binding.into_cmdlist(),
        );
        CMD_RETURN_NORMAL
    }
}
