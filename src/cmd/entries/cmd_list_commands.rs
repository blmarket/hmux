//! `list-commands`: the table of every command the server knows, printed one
//! line per command through the format engine.
//!
//! With no argument the whole table is walked in the order it is declared;
//! with one, `cmd_find` resolves it by name or alias and only that command is
//! printed, an unknown name being the command's one error. Each line is the
//! `-F` template, or the built-in one, expanded against three variables the
//! entry supplies — its name, its alias and its usage — and a line that
//! expands to nothing is not printed at all.
//!
//! The command table stays the array the rest of the crate reads, walked to
//! the null it ends with.

use crate::args::args_parse_t;
use crate::cmd::cmd_get_args;

use crate::cmd::cmdq_item;
use crate::cmd::{RustCommandEntry, cmd, cmd_entry_flag, cmd_retval};
use crate::consts::{
    CMD_AFTERHOOK, CMD_FIND_PANE, CMD_RETURN_ERROR, CMD_RETURN_NORMAL, CMD_STARTSERVER, FORMAT_NONE,
};
use crate::fmt_args;
use crate::format::{
    format_add, format_create_for_client, format_defaults_for_handles, format_expand,
};
use crate::types::format_tree;
use crate::{CommandCatalog, CommandEntry, RustCommandCatalog};
use ::core::ffi::CStr;

pub const LIST_COMMANDS_TEMPLATE: &CStr =
    c"#{command_list_name}#{?command_list_alias, (#{command_list_alias}),} #{command_list_usage}";
pub(crate) static cmd_list_commands_entry: RustCommandEntry = RustCommandEntry {
    name: c"list-commands",
    alias: Some(c"lscm"),
    args: args_parse_t {
        template: c"F:",
        lower: 0,
        upper: 1,
        cb: None,
    },
    usage: c"[-F format] [command]",
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
    flags: CMD_STARTSERVER | CMD_AFTERHOOK,
    exec: cmd_list_commands,
};

/// The text an entry carries under its alias, as the empty string when it
/// carries none — which is what the format engine is handed for a command
/// with no alias.
fn text_or_empty(s: Option<&'static CStr>) -> &'static CStr {
    s.unwrap_or(c"")
}

/// Prints the one line `entry` gives under `template`, unless expanding it
/// left nothing to print.
unsafe fn cmd_list_single_command(
    entry: &RustCommandEntry,
    ft: &mut format_tree,
    template: &CStr,
    item: &cmdq_item,
) {
    unsafe {
        format_add(ft, c"command_list_name", c"%s", fmt_args![entry.name()]);
        format_add(
            ft,
            c"command_list_alias",
            c"%s",
            fmt_args![text_or_empty(entry.alias())],
        );
        format_add(ft, c"command_list_usage", c"%s", fmt_args![entry.usage()]);

        let line = format_expand(ft, template);
        if !line.as_bytes().is_empty() {
            item.print(c"%s", fmt_args![line.as_c_str()]);
        }
    }
}

unsafe fn cmd_list_commands(self_0: &cmd, item: &cmdq_item) -> cmd_retval {
    let args = cmd_get_args(self_0);
    let template = args
        .argument_flag_string(b'F')
        .unwrap_or(LIST_COMMANDS_TEMPLATE);

    let mut ft = format_create_for_client(item.client().as_ref(), Some(item), FORMAT_NONE, 0);
    unsafe { format_defaults_for_handles(&mut ft, None, None, None, None) };

    let catalog = RustCommandCatalog;
    let command = args.argument_string(0);
    if let Some(command) = command {
        let entry = match catalog.find(command) {
            Ok(entry) => entry,
            Err(cause) => {
                unsafe { item.error(c"%s", fmt_args![cause.as_c_str()]) };
                return CMD_RETURN_ERROR;
            }
        };
        unsafe { cmd_list_single_command(entry, &mut ft, template, item) };
    } else {
        for &entry in catalog.entries() {
            unsafe { cmd_list_single_command(entry, &mut ft, template, item) };
        }
    }

    CMD_RETURN_NORMAL
}
