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

use crate::args::RustArguments;
use crate::cmd::cmd_get_args;
use crate::cmd::parse::{cmd_parse_from_arguments, cmd_parse_from_string};

use crate::cmd::cmdq_item;
use crate::cmd::{CmdListRef, RustCommandEntry, cmd, cmd_entry_flag, cmd_retval};
use crate::consts::{
    ARGS_PARSE_COMMANDS_OR_STRING, CMD_AFTERHOOK, CMD_FIND_PANE, CMD_PARSE_ERROR, CMD_RETURN_ERROR,
    CMD_RETURN_NORMAL, KEYC_NONE, KEYC_UNKNOWN,
};
use crate::fmt_args;
use crate::key_bindings::key_bindings_add;
use crate::text::{KeyStringCodec, RustKeyStringCodec};
use crate::types::{ArgsValue, args, args_parse_t, args_parse_type, u_int};
use ::core::ffi::CStr;
use ::std::ffi::CString;

pub(crate) static cmd_bind_key_entry: RustCommandEntry = RustCommandEntry {
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
fn table_name(args: &RustArguments) -> &CStr {
    if let Some(name) = args.argument_flag_string(b'T') {
        name
    } else if args.argument_flag_count(b'n') != 0 {
        c"root"
    } else {
        c"prefix"
    }
}

/// Reads what the line binds to the key, answering the parser's error message
/// — which the caller gives back — when the words after the key are not a
/// command.
unsafe fn binding_of(args: &RustArguments, count: u_int) -> Result<Binding, CString> {
    unsafe {
        if count == 1 {
            return Ok(Binding::Keep);
        }
        let value = args.argument_value(1).expect("the binding has a command argument");
        if count == 2
            && let ArgsValue::Commands { cmdlist, .. } = value
        {
            return Ok(Binding::Shared(cmdlist.clone().unwrap()));
        }
        let mut pr = if count == 2 {
            cmd_parse_from_string(
                args.argument_string(1).expect("argument count checked"),
                None,
            )
        } else {
            cmd_parse_from_arguments(&args.argument_values()[1..], None)
        };
        if pr.status == CMD_PARSE_ERROR {
            return Err(pr.error.take().unwrap());
        }
        Ok(Binding::Owned(pr.cmdlist.take().unwrap()))
    }
}

fn cmd_bind_key_args_parse(
    _args: &args,
    _idx: u_int,
    _cause: &mut Option<CString>,
) -> args_parse_type {
    ARGS_PARSE_COMMANDS_OR_STRING
}

unsafe fn cmd_bind_key_exec(self_0: &cmd, item: &cmdq_item) -> cmd_retval {
    let args = cmd_get_args(self_0);
    let note = args.argument_flag_string(b'N');
    let count = args.argument_count();

    let keyname = unsafe { args.argument_string(0).expect("argument count checked") };
    let key = RustKeyStringCodec.parse_key(keyname);
    if key == KEYC_NONE || key == KEYC_UNKNOWN {
        unsafe { item.error(c"unknown key: %s", fmt_args![keyname]) };
        return CMD_RETURN_ERROR;
    }

    let tablename = table_name(args);
    let repeat = args.argument_flag_count(b'r');

    let binding = match unsafe { binding_of(args, count) } {
        Ok(binding) => binding,
        Err(error) => {
            unsafe { item.error(c"%s", fmt_args![error.as_c_str()]) };
            return CMD_RETURN_ERROR;
        }
    };
    unsafe { key_bindings_add(tablename, key, note, repeat, binding.into_cmdlist()) };
    CMD_RETURN_NORMAL
}
