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
//!
//! Coverage exemptions: none. The message-protocol, enumeration, style,
//! prompt, sorting and mouse-key constants below are not this module's own,
//! but the tests pin their values through it, so they stay where the
//! transpiler put them.
use crate::arguments::{args_get, args_get_str, args_has, args_string};
use crate::cmd::queue::{cmdq_error, cmdq_get_client, cmdq_get_target_client, cmdq_print};
use crate::cmd::{cmd_get_args, cmd_list_print};
use crate::fmt_args;
use crate::format::{format_add, format_create, format_defaults, format_expand};
use crate::key_bindings::{
    key_binding_cmdlist, key_binding_flags, key_binding_key, key_binding_note,
    key_binding_tablename, key_bindings_get_table, key_bindings_has_repeat,
};
use crate::options::options_get_number;
use crate::sort::sort_order_from_string;
use crate::sort::{sort_get_key_bindings, sort_get_key_bindings_table};
use crate::status::status_message_set;
use crate::text::utf8_cstrwidth;
use crate::text::{key_string_lookup_key, key_string_lookup_string};
use crate::tmux::global_s_options;
pub use crate::types::*;
use ::core::ffi::{c_char, c_int};
use ::core::ptr::null_mut;
use ::std::ffi::{CStr, CString};
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
pub type keyc = ::core::ffi::c_ulong;
pub const KEYC_TRIPLECLICK_CONTROL0: keyc = 51539607561;
pub const KEYC_TRIPLECLICK_SCROLLBAR_DOWN: keyc = 51539607560;
pub const KEYC_TRIPLECLICK_SCROLLBAR_SLIDER: keyc = 51539607559;
pub const KEYC_TRIPLECLICK_SCROLLBAR_UP: keyc = 51539607558;
pub const KEYC_TRIPLECLICK_BORDER: keyc = 51539607557;
pub const KEYC_TRIPLECLICK_STATUS: keyc = 51539607553;
pub const KEYC_TRIPLECLICK_PANE: keyc = 51539607552;
pub const KEYC_DOUBLECLICK_CONTROL0: keyc = 47244640265;
pub const KEYC_DOUBLECLICK_SCROLLBAR_DOWN: keyc = 47244640264;
pub const KEYC_DOUBLECLICK_PANE: keyc = 47244640256;
pub const KEYC_SECONDCLICK_PANE: keyc = 42949672960;
pub const KEYC_MOUSEDRAGEND_PANE: keyc = 30064771072;
pub const KEYC_MOUSEDRAG_PANE: keyc = 25769803776;
pub const KEYC_PASTE_END: keyc = 8589934598;
pub const KEYC_PASTE_START: keyc = 8589934597;
pub const KEYC_ANY: keyc = 8589934596;
pub const KEYC_FOCUS_OUT: keyc = 8589934595;
pub const KEYC_FOCUS_IN: keyc = 8589934594;
pub const KEYC_UNKNOWN: keyc = 8589934593;
pub const KEYC_NONE: keyc = 8589934592;
pub const KEYC_USER: keyc = 4294967296;
pub const ARGS_PARSE_COMMANDS: args_parse_type = 3;
pub const ARGS_PARSE_COMMANDS_OR_STRING: args_parse_type = 2;
pub const ARGS_PARSE_STRING: args_parse_type = 1;
pub const ARGS_PARSE_INVALID: args_parse_type = 0;
pub const CMD_FIND_SESSION: cmd_find_type = 2;
pub const CMD_FIND_WINDOW: cmd_find_type = 1;
pub const CMD_FIND_PANE: cmd_find_type = 0;
pub const CMD_RETURN_STOP: cmd_retval = 2;
pub const CMD_RETURN_WAIT: cmd_retval = 1;
pub const CMD_RETURN_NORMAL: cmd_retval = 0;
pub const CMD_RETURN_ERROR: cmd_retval = -1;
pub const SORT_END: sort_order = 8;
pub const SORT_Z: sort_order = 7;
pub const SORT_SIZE: sort_order = 6;
pub const SORT_ORDER: sort_order = 5;
pub const SORT_NAME: sort_order = 4;
pub const SORT_MODIFIER: sort_order = 3;
pub const SORT_INDEX: sort_order = 2;
pub const SORT_CREATION: sort_order = 1;
pub const SORT_ACTIVITY: sort_order = 0;
pub const KEYC_MASK_MODIFIERS: ::core::ffi::c_ulonglong =
    0xff0000000000 as ::core::ffi::c_ulonglong;
pub const KEYC_MASK_KEY: ::core::ffi::c_ulonglong = 0xffffffffff as ::core::ffi::c_ulonglong;
pub const CMD_STARTSERVER: ::core::ffi::c_int = 0x1 as ::core::ffi::c_int;
pub const CMD_AFTERHOOK: ::core::ffi::c_int = 0x4 as ::core::ffi::c_int;
pub const KEY_BINDING_REPEAT: ::core::ffi::c_int = 0x1 as ::core::ffi::c_int;
pub const FORMAT_NONE: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const CMD_LIST_PRINT_ESCAPED: ::core::ffi::c_int = 0x1 as ::core::ffi::c_int;
pub const CMD_LIST_PRINT_NO_GROUPS: ::core::ffi::c_int = 0x2 as ::core::ffi::c_int;
pub const LIST_KEYS_TEMPLATE: [::core::ffi::c_char; 250] = unsafe {
    ::core::mem::transmute::<
        [u8; 250],
        [::core::ffi::c_char; 250],
    >(
        *b"#{?notes_only,#{key_prefix} #{p|#{key_string_width}:key_string} #{?key_note,#{key_note},#{key_command}},bind-key #{?key_has_repeat,#{?key_repeat,-r,  },} -T #{p|#{key_table_width}:key_table} #{p|#{key_string_width}:#{q|a:key_string}} #{key_command}}\0",
    )
};
pub(crate) static cmd_list_keys_entry: cmd_entry = cmd_entry {
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
unsafe fn cmd_list_keys_get_prefix(args: &args) -> CString {
    unsafe {
        if args_has(args, b'P') != 0 {
            return CStr::from_ptr(args_get(args, b'P')).to_owned();
        }
        let prefix = options_get_number(global_s_options, c"prefix".as_ptr()) as key_code;
        if prefix == KEYC_NONE {
            return CString::default();
        }
        CStr::from_ptr(key_string_lookup_key(prefix, 0)).to_owned()
    }
}

/// The width of the widest key name in the list, which is the column every
/// key name is padded into.
unsafe fn cmd_list_keys_get_width(l: &[*mut key_binding]) -> u_int {
    unsafe {
        l.iter()
            .map(|&bd| utf8_cstrwidth(key_string_lookup_key(key_binding_key(bd), 0)))
            .max()
            .unwrap_or(0)
    }
}

/// The width of the widest table name in the list, which is the column every
/// table name is padded into.
unsafe fn cmd_list_keys_get_table_width(l: &[*mut key_binding]) -> u_int {
    unsafe {
        l.iter()
            .map(|&bd| key_binding_tablename(bd).map_or(0, |name| utf8_cstrwidth(name.as_ptr())))
            .max()
            .unwrap_or(0)
    }
}

/// The `prefix` and `root` tables sorted and laid end to end, in that order.
unsafe fn cmd_list_keys_get_root_and_prefix(sort_crit: &sort_criteria_t) -> Vec<*mut key_binding> {
    unsafe {
        let mut l = Vec::new();
        for name in [c"prefix", c"root"] {
            let t = key_bindings_get_table(name.as_ptr(), 0);
            let lt = sort_get_key_bindings_table(t, sort_crit);
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
unsafe fn cmd_list_keys_filter_key_list(
    filter_notes: c_int,
    filter_key: c_int,
    only: key_code,
    l: &mut [*mut key_binding],
) -> u_int {
    unsafe {
        let mut j = 0;
        for i in 0..l.len() {
            let bd = l[i];
            let key = key_binding_key(bd) & (KEYC_MASK_KEY | KEYC_MASK_MODIFIERS);
            if filter_key != 0 && only != key {
                continue;
            }
            if filter_notes != 0 && key_binding_note(bd).is_none() {
                continue;
            }
            l[j] = bd;
            j += 1;
        }
        j as u_int
    }
}

/// The six variables one binding gives the template: whether it repeats, the
/// note it carries as the empty string when it carries none, the prefix
/// string, its table, its key and the command list it runs.
unsafe fn cmd_list_keys_format_add_key_binding(
    ft: &mut format_tree,
    bd: *const key_binding,
    prefix: *const c_char,
) {
    unsafe {
        if key_binding_flags(bd) & KEY_BINDING_REPEAT != 0 {
            format_add(ft, c"key_repeat", c"1".as_ptr(), fmt_args![]);
        } else {
            format_add(ft, c"key_repeat", c"0".as_ptr(), fmt_args![]);
        }

        let note = key_binding_note(bd).unwrap_or(c"");
        format_add(
            ft,
            c"key_note",
            c"%s".as_ptr(),
            fmt_args![note.as_ptr()],
        );

        format_add(ft, c"key_prefix", c"%s".as_ptr(), fmt_args![prefix]);
        format_add(
            ft,
            c"key_table",
            c"%s".as_ptr(),
            fmt_args![key_binding_tablename(bd)],
        );
        format_add(
            ft,
            c"key_string",
            c"%s".as_ptr(),
            fmt_args![key_string_lookup_key(key_binding_key(bd), 0)],
        );

        let s = key_binding_cmdlist(bd).map_or_else(CString::default, |cmdlist| {
            cmd_list_print(cmdlist, CMD_LIST_PRINT_ESCAPED | CMD_LIST_PRINT_NO_GROUPS)
        });
        format_add(
            ft,
            c"key_command",
            c"%s".as_ptr(),
            fmt_args![s.as_ptr()],
        );
    }
}

unsafe fn cmd_list_keys_exec(self_0: &cmd, item: *mut cmdq_item) -> cmd_retval {
    unsafe {
        let args = cmd_get_args(self_0);
        let tc = cmdq_get_target_client(&*item);
        let mut table: *mut key_table = null_mut();
        let mut only: key_code = KEYC_UNKNOWN;

        let keystr = args_string(args, 0);
        if !keystr.is_null() {
            only = key_string_lookup_string(keystr);
            if only == KEYC_UNKNOWN {
                cmdq_error(item, c"invalid key: %s".as_ptr(), fmt_args![keystr]);
                return CMD_RETURN_ERROR;
            }
            only &= KEYC_MASK_KEY | KEYC_MASK_MODIFIERS;
        }

        let mut sort_crit = sort_criteria_t {
            order: sort_order_from_string(args_get_str(args, b'O')),
            ..Default::default()
        };
        if sort_crit.order == SORT_END && args_has(args, b'O') != 0 {
            cmdq_error(item, c"invalid sort order".as_ptr(), fmt_args![]);
            return CMD_RETURN_ERROR;
        }
        sort_crit.reversed = args_has(args, b'r');

        let tablename = args_get(args, b'T');
        if !tablename.is_null() {
            table = key_bindings_get_table(tablename, 0);
            if table.is_null() {
                cmdq_error(
                    item,
                    c"table %s doesn't exist".as_ptr(),
                    fmt_args![tablename],
                );
                return CMD_RETURN_ERROR;
            }
        }

        let prefix = cmd_list_keys_get_prefix(args);
        let single = args_has(args, b'1');
        let notes_only = args_has(args, b'N');

        let mut template = args_get(args, b'F');
        if template.is_null() {
            template = LIST_KEYS_TEMPLATE.as_ptr();
        }

        let mut l = if !table.is_null() {
            sort_get_key_bindings_table(table, &sort_crit)
        } else if notes_only != 0 {
            cmd_list_keys_get_root_and_prefix(&sort_crit)
        } else {
            sort_get_key_bindings(&sort_crit)
        };

        let filter_notes = (notes_only != 0 && args_has(args, b'a') == 0) as c_int;
        let filter_key = (only != KEYC_UNKNOWN) as c_int;
        let mut n = if filter_notes != 0 || filter_key != 0 {
            cmd_list_keys_filter_key_list(filter_notes, filter_key, only, &mut l)
        } else {
            l.len() as u_int
        };
        if single != 0 {
            n = 1;
        }

        let mut ft = format_create(cmdq_get_client(&*item), item, FORMAT_NONE, 0);
        format_defaults(&mut ft, null_mut(), null_mut(), null_mut(), null_mut());
        format_add(
            &mut ft,
            c"notes_only",
            c"%d".as_ptr(),
            fmt_args![notes_only],
        );
        format_add(
            &mut ft,
            c"key_has_repeat",
            c"%d".as_ptr(),
            fmt_args![key_bindings_has_repeat(&l[..n as usize])],
        );
        format_add(
            &mut ft,
            c"key_string_width",
            c"%u".as_ptr(),
            fmt_args![cmd_list_keys_get_width(&l[..n as usize])],
        );
        format_add(
            &mut ft,
            c"key_table_width",
            c"%u".as_ptr(),
            fmt_args![cmd_list_keys_get_table_width(&l[..n as usize])],
        );

        for &bd in &l[..n as usize] {
            cmd_list_keys_format_add_key_binding(&mut ft, bd, prefix.as_ptr());

            let line = format_expand(&mut ft, CStr::from_ptr(template));
            if single != 0 && !tc.is_null() || n == 1 {
                status_message_set(tc, -1, 1, 0, 0, c"%s".as_ptr(), fmt_args![line.as_ptr()]);
            } else if !line.as_bytes().is_empty() {
                cmdq_print(item, c"%s".as_ptr(), fmt_args![line.as_ptr()]);
            }

            if single != 0 {
                break;
            }
        }

        CMD_RETURN_NORMAL
    }
}

#[cfg(test)]
#[path = "../tests/test_cmd_list_keys.rs"]
mod tests;
