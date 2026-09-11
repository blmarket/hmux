//! `list-keys`: the key bindings the server holds, printed one line each
//! through the format engine.
//!
//! Which bindings are listed is decided first: `-T` names one key table and
//! lists only that, `-N` merges the `prefix` and `root` tables and keeps only
//! the bindings that carry a note unless `-a` asks for all of them, and
//! otherwise every table is listed. A key given as the one argument narrows
//! whatever that produced to the bindings whose key and modifiers match it,
//! and `-1` cuts the list to its first line. `-O` picks the order the `sort`
//! module applies and `-r` reverses it.
//!
//! Each line is the `-F` template, or the built-in one, expanded against the
//! six per-binding variables and the four list-wide ones the hook adds: the
//! `-N` marker, whether any binding in the list repeats, and the two column
//! widths that pad the key and table names into place. `-P` supplies the
//! prefix string the `-N` form starts each line with; without it the name of
//! the `prefix` key is used, and the empty string when there is no prefix key.
//!
//! Where a line goes depends on how many there are: a single line — because
//! `-1` was given and the item has a target client, or because the list came
//! down to one binding — is set as that client's status message, and every
//! other line is printed, unless it expanded to nothing.
//!
//! The merged `-N` list lives in one buffer that is reused between calls, as
//! the C's `static` array was; the sorted lists the `sort` module hands back
//! are buffers of its own, which is why the merge copies out of them, and why
//! the filter compacts a list in place the way the C did.
//!
//! Upstream quirk kept: `-1` sets the count to one whatever the filtering
//! left, so a `-1` that matched no binding still lists the entry the filter
//! left sitting in the front slot — the binding that was first before the
//! filter ran.

use crate::args::arguments_trait::Arguments as _;
use crate::args::RustArguments;
use crate::args::args_parse_t;
use crate::cmd::cmd_get_args;

use crate::fmt_args;
use crate::format::{
    format_add, format_create_for_client, format_defaults_for_handles, format_expand,
};
use crate::key_bindings::{
    key_binding, key_binding_cmdlist, key_binding_flags, key_binding_key, key_binding_note,
    key_binding_tablename, key_bindings_get_table, key_bindings_has_repeat,
};

use crate::cmd::cmdq_item;
use crate::cmd::{RustCommandEntry, cmd, cmd_entry_flag, cmd_retval};
use crate::consts::{
    CMD_AFTERHOOK, CMD_FIND_PANE, CMD_LIST_PRINT_ESCAPED, CMD_LIST_PRINT_NO_GROUPS,
    CMD_RETURN_ERROR, CMD_RETURN_NORMAL, CMD_STARTSERVER, FORMAT_NONE, KEY_BINDING_REPEAT,
    KEYC_MASK_KEY, KEYC_MASK_MODIFIERS, KEYC_NONE, KEYC_UNKNOWN, SORT_END,
};
use crate::sort::sort_get_key_bindings;
use crate::sort::{RustSortCriteria, SortCriteria};
use crate::status::status_message_for_client;
use crate::text::{KeyStringCodec, RustKeyStringCodec, RustUtf8VisModel, Utf8VisModel};
use crate::tmux::global_session_options;
use crate::types::{OptionsRef, format_tree, key_code, sort_criteria_t, u_int};
use ::core::ffi::c_int;
use ::std::ffi::{CStr, CString};

pub const LIST_KEYS_TEMPLATE: &CStr = c"#{?notes_only,#{key_prefix} #{p|#{key_string_width}:key_string} #{?key_note,#{key_note},#{key_command}},bind-key #{?key_has_repeat,#{?key_repeat,-r,  },} -T #{p|#{key_table_width}:key_table} #{p|#{key_string_width}:#{q|a:key_string}} #{key_command}}";
pub(crate) static cmd_list_keys_entry: RustCommandEntry = RustCommandEntry {
    name: c"list-keys",
    alias: Some(c"lsk"),
    args: args_parse_t {
        template: c"1aF:NO:P:rT:",
        lower: 0,
        upper: 1,
        cb: None,
    },
    usage: c"[-1aNr] [-F format] [-O order] [-P prefix-string][-T key-table] [key]",
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
    exec: cmd_list_keys_exec,
};

/// The string every `-N` line starts with: the `-P` argument as given, the
/// name of the `prefix` key, or nothing at all when there is no prefix key.
/// The answer is freshly allocated and owned by the caller.
unsafe fn cmd_list_keys_get_prefix(args: &RustArguments) -> CString {
    unsafe {
        if let Some(given) = args.argument_flag_string(b'P') {
            return given.to_owned();
        }
        let prefix = (global_session_options()
            .as_ref()
            .expect("global options are initialized"))
        .number(c"prefix") as key_code;
        if prefix == KEYC_NONE {
            return CString::default();
        }
        RustKeyStringCodec.format_key(prefix, false)
    }
}

/// The width of the widest key name in the list, which is the column every
/// key name is padded into.
fn cmd_list_keys_get_width(l: &[key_binding]) -> u_int {
    l.iter()
        .map(|bd| {
            let key_name = RustKeyStringCodec.format_key(key_binding_key(bd), false);
            RustUtf8VisModel.width(&key_name)
        })
        .max()
        .unwrap_or(0)
}

/// The width of the widest table name in the list, which is the column every
/// table name is padded into.
fn cmd_list_keys_get_table_width(l: &[key_binding]) -> u_int {
    l.iter()
        .map(|bd| key_binding_tablename(bd).map_or(0, |name| RustUtf8VisModel.width(name)))
        .max()
        .unwrap_or(0)
}

/// The `prefix` and `root` tables sorted and laid end to end, in that order.
fn cmd_list_keys_get_root_and_prefix(sort_crit: &sort_criteria_t) -> Vec<key_binding> {
    {
        let mut l = Vec::new();
        for name in [c"prefix", c"root"] {
            let Some(t) = key_bindings_get_table(name, 0) else {
                continue;
            };
            let lt = t.sorted_bindings(sort_crit);
            l.extend(lt);
        }
        l
    }
}

/// Keeps in `l` only the bindings that pass the filters asked for — the key
/// and its modifiers matching `only`, and a note being present — compacting
/// them into the front of the list and answering how many are left. What sits
/// past that count is whatever was there before, which is what makes the `-1`
/// quirk below visible.
fn cmd_list_keys_filter_key_list(
    filter_notes: c_int,
    filter_key: c_int,
    only: key_code,
    l: &mut [key_binding],
) -> u_int {
    let mut j = 0;
    for i in 0..l.len() {
        let key = key_binding_key(&l[i]) & (KEYC_MASK_KEY | KEYC_MASK_MODIFIERS);
        if filter_key != 0 && only != key {
            continue;
        }
        if filter_notes != 0 && key_binding_note(&l[i]).is_none() {
            continue;
        }
        if i != j {
            l[j] = l[i].clone();
        }
        j += 1;
    }
    j as u_int
}

/// The six variables one binding gives the template: whether it repeats, the
/// note it carries as the empty string when it carries none, the prefix
/// string, its table, its key and the command list it runs.
unsafe fn cmd_list_keys_format_add_key_binding(
    ft: &mut format_tree,
    bd: &key_binding,
    prefix: &CStr,
) {
    unsafe {
        if key_binding_flags(bd) & KEY_BINDING_REPEAT != 0 {
            format_add(ft, c"key_repeat", c"1", fmt_args![]);
        } else {
            format_add(ft, c"key_repeat", c"0", fmt_args![]);
        }

        let note = key_binding_note(bd).unwrap_or(c"");
        format_add(ft, c"key_note", c"%s", fmt_args![note]);

        format_add(ft, c"key_prefix", c"%s", fmt_args![prefix]);
        format_add(
            ft,
            c"key_table",
            c"%s",
            fmt_args![key_binding_tablename(bd)],
        );
        let key_name = RustKeyStringCodec.format_key(key_binding_key(bd), false);
        format_add(ft, c"key_string", c"%s", fmt_args![key_name.as_c_str()]);

        let s = key_binding_cmdlist(bd).map_or_else(CString::default, |cmdlist| {
            cmdlist.print(CMD_LIST_PRINT_ESCAPED | CMD_LIST_PRINT_NO_GROUPS)
        });
        format_add(ft, c"key_command", c"%s", fmt_args![s.as_c_str()]);
    }
}

unsafe fn cmd_list_keys_exec(self_0: &cmd, item: &cmdq_item) -> cmd_retval {
    let args = cmd_get_args(self_0);
    let mut tc = item.target_client();
    let mut table = None;
    let mut only: key_code = KEYC_UNKNOWN;

    let keystr = args.argument_string(0);
    if let Some(keystr) = keystr {
        only = RustKeyStringCodec.parse_key(keystr);
        if only == KEYC_UNKNOWN {
            unsafe { item.error(c"invalid key: %s", fmt_args![keystr]) };
            return CMD_RETURN_ERROR;
        }
        only &= KEYC_MASK_KEY | KEYC_MASK_MODIFIERS;
    }

    let mut sort_crit = RustSortCriteria::new(
        RustSortCriteria::parse_order(args.argument_flag_string(b'O')),
        false,
    );
    if sort_crit.order() == SORT_END && args.argument_flag_count(b'O') != 0 {
        unsafe { item.error(c"invalid sort order", fmt_args![]) };
        return CMD_RETURN_ERROR;
    }
    sort_crit.set_reversed(args.argument_flag_count(b'r') != 0);

    let tablename = args.argument_flag_string(b'T');
    if let Some(tablename) = tablename {
        table = key_bindings_get_table(tablename, 0);
        if table.is_none() {
            unsafe { item.error(c"table %s doesn't exist", fmt_args![tablename]) };
            return CMD_RETURN_ERROR;
        }
    }

    let prefix = unsafe { cmd_list_keys_get_prefix(args) };
    let single = args.argument_flag_count(b'1');
    let notes_only = args.argument_flag_count(b'N');

    let template = args
        .argument_flag_string(b'F')
        .unwrap_or(LIST_KEYS_TEMPLATE);

    let mut l = if let Some(table) = table {
        table.sorted_bindings(&sort_crit)
    } else if notes_only != 0 {
        cmd_list_keys_get_root_and_prefix(&sort_crit)
    } else {
        sort_get_key_bindings(&sort_crit)
    };

    let filter_notes = (notes_only != 0 && args.argument_flag_count(b'a') == 0) as c_int;
    let filter_key = (only != KEYC_UNKNOWN) as c_int;
    let mut n = if filter_notes != 0 || filter_key != 0 {
        cmd_list_keys_filter_key_list(filter_notes, filter_key, only, &mut l)
    } else {
        l.len() as u_int
    };
    if single != 0 {
        n = 1;
    }

    let mut ft = format_create_for_client(item.client().as_ref(), Some(item), FORMAT_NONE, 0);
    unsafe { format_defaults_for_handles(&mut ft, None, None, None, None) };
    format_add(&mut ft, c"notes_only", c"%d", fmt_args![notes_only]);
    format_add(
        &mut ft,
        c"key_has_repeat",
        c"%d",
        fmt_args![key_bindings_has_repeat(&l[..n as usize])],
    );
    format_add(
        &mut ft,
        c"key_string_width",
        c"%u",
        fmt_args![cmd_list_keys_get_width(&l[..n as usize])],
    );
    format_add(
        &mut ft,
        c"key_table_width",
        c"%u",
        fmt_args![cmd_list_keys_get_table_width(&l[..n as usize])],
    );

    for bd in &l[..n as usize] {
        unsafe { cmd_list_keys_format_add_key_binding(&mut ft, bd, &prefix) };

        let line = unsafe { format_expand(&mut ft, template) };
        if single != 0 && tc.is_some() || n == 1 {
            unsafe {
                status_message_for_client(
                    tc.as_mut(),
                    -1,
                    1,
                    0,
                    0,
                    c"%s",
                    fmt_args![line.as_c_str()],
                )
            };
        } else if !line.as_bytes().is_empty() {
            unsafe { item.print(c"%s", fmt_args![line.as_c_str()]) };
        }

        if single != 0 {
            break;
        }
    }

    CMD_RETURN_NORMAL
}

#[cfg(test)]
#[path = "../../tests/test_cmd_list_keys.rs"]
mod tests;
