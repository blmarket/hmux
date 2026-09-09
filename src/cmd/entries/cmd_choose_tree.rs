//! `choose-tree`, `choose-client`, `choose-buffer` and `customize-mode`: the
//! four commands that put one of the mode-tree browsers on a pane.
//!
//! All four share one exec routine and differ only in which
//! [`WindowMode`](WindowMode) they hand
//! [`window_pane_set_mode`], which is picked by comparing the command's own
//! entry against the three named statics — anything else, `choose-tree`
//! included, opens the window tree. The arguments the command was given are
//! passed straight through to the mode, which is what reads `-F`, `-K`, `-O`,
//! the `-s`/`-w` tree level and the template.
//!
//! Quirks kept:
//!
//! * `-O` is parsed here only to be thrown away. The order the parse answers
//!   is never used — the mode tree reads `-O` out of the same arguments and
//!   parses it a second time — so all this routine does with it is refuse a
//!   name no order goes by.
//! * That refusal comes first, before either "nothing to choose from" check,
//!   so `choose-buffer -O bogus` reports the bad order even when the paste
//!   store is empty and no mode would have been opened.
//! * `choose-buffer` with an empty paste store and `choose-client` with no
//!   clients answer success and open nothing at all, rather than reporting
//!   that there is nothing to choose.
//! * `customize-mode` shares the `-O` check although its own template has no
//!   `O`, so the check can never fire for it: the parser turns the flag down
//!   first and `args_has` is what guards the refusal.

use crate::args::{args_get_str, args_has};

use crate::cmd::{cmd_get_args, cmd_get_entry};
use crate::fmt_args;
use crate::paste::{PasteBufferStore, with_paste_buffers};
use crate::server::server_client_how_many;
use crate::sort::{RustSortCriteria, SortCriteria};
use crate::cmd::{RustCommandEntry, cmd, cmd_entry_flag, cmd_retval};
use crate::cmd::cmdq_item;
use crate::types::{WindowMode, args, args_parse_t, args_parse_type, u_int};
use ::core::ffi::c_char;
use ::std::ffi::CString;

use crate::consts::{
    ARGS_PARSE_COMMANDS_OR_STRING, CMD_FIND_PANE, CMD_RETURN_ERROR, CMD_RETURN_NORMAL, SORT_END,
};

pub(crate) static cmd_choose_tree_entry: RustCommandEntry = RustCommandEntry {
    name: c"choose-tree",
    alias: None,
    args: args_parse_t {
        template: c"F:f:GK:NO:rst:wyZ",
        lower: 0,
        upper: 1,
        cb: Some(
            cmd_choose_tree_args_parse,
        ),
    },
    usage: c"[-GNrswZ] [-F format] [-f filter] [-K key-format] [-O sort-order] [-t target-pane] [template]"
        ,
    source: cmd_entry_flag {
        flag: 0,
        type_0: CMD_FIND_PANE,
        flags: 0,
    },
    target: cmd_entry_flag {
        flag: b't' as c_char,
        type_0: CMD_FIND_PANE,
        flags: 0,
    },
    flags: 0,
    exec: cmd_choose_tree_exec,
};

pub(crate) static cmd_choose_client_entry: RustCommandEntry = RustCommandEntry {
    name: c"choose-client",
    alias: None,
    args: args_parse_t {
        template: c"F:f:K:NO:rt:yZ",
        lower: 0,
        upper: 1,
        cb: Some(
            cmd_choose_tree_args_parse,
        ),
    },
    usage: c"[-NrZ] [-F format] [-f filter] [-K key-format] [-O sort-order] [-t target-pane] [template]"
        ,
    source: cmd_entry_flag {
        flag: 0,
        type_0: CMD_FIND_PANE,
        flags: 0,
    },
    target: cmd_entry_flag {
        flag: b't' as c_char,
        type_0: CMD_FIND_PANE,
        flags: 0,
    },
    flags: 0,
    exec: cmd_choose_tree_exec,
};

pub(crate) static cmd_choose_buffer_entry: RustCommandEntry = RustCommandEntry {
    name: c"choose-buffer",
    alias: None,
    args: args_parse_t {
        template: c"F:f:K:NO:rt:yZ",
        lower: 0,
        upper: 1,
        cb: Some(
            cmd_choose_tree_args_parse,
        ),
    },
    usage: c"[-NrZ] [-F format] [-f filter] [-K key-format] [-O sort-order] [-t target-pane] [template]"
        ,
    source: cmd_entry_flag {
        flag: 0,
        type_0: CMD_FIND_PANE,
        flags: 0,
    },
    target: cmd_entry_flag {
        flag: b't' as c_char,
        type_0: CMD_FIND_PANE,
        flags: 0,
    },
    flags: 0,
    exec: cmd_choose_tree_exec,
};

pub(crate) static cmd_customize_mode_entry: RustCommandEntry = RustCommandEntry {
    name: c"customize-mode",
    alias: None,
    args: args_parse_t {
        template: c"F:f:Nt:yZ",
        lower: 0,
        upper: 0,
        cb: None,
    },
    usage: c"[-NZ] [-F format] [-f filter] [-t target-pane]",
    source: cmd_entry_flag {
        flag: 0,
        type_0: CMD_FIND_PANE,
        flags: 0,
    },
    target: cmd_entry_flag {
        flag: b't' as c_char,
        type_0: CMD_FIND_PANE,
        flags: 0,
    },
    flags: 0,
    exec: cmd_choose_tree_exec,
};

/// How the parser is told to read the template the three tree commands take:
/// as a command list if it parses as one, and as a plain string otherwise.
fn cmd_choose_tree_args_parse(
    _args: &args,
    _idx: u_int,
    _cause: &mut Option<CString>,
) -> args_parse_type {
    ARGS_PARSE_COMMANDS_OR_STRING
}

/// The mode the entry behind `self_0` opens, or nothing when there is nothing
/// to choose from — an empty paste store for `choose-buffer`, no clients at
/// all for `choose-client` — which ends the command having opened no mode.
///
/// The comparison is against the entry statics themselves, so `choose-tree`
/// and any future entry sharing this exec fall through to the window tree.
fn cmd_choose_tree_mode(self_0: &cmd) -> Option<WindowMode> {
    let entry = cmd_get_entry(self_0);
    if core::ptr::eq(entry, &cmd_choose_buffer_entry) {
        match with_paste_buffers(PasteBufferStore::is_empty) {
            false => Some(WindowMode::Buffer),
            true => None,
        }
    } else if core::ptr::eq(entry, &cmd_choose_client_entry) {
        match server_client_how_many() {
            0 => None,
            _ => Some(WindowMode::Client),
        }
    } else if core::ptr::eq(entry, &cmd_customize_mode_entry) {
        Some(WindowMode::Customize)
    } else {
        Some(WindowMode::Tree)
    }
}

/// Whether the `-O` the command carries names an order. A command with no `-O`
/// is fine whatever the parse answered, which is what keeps `customize-mode`,
/// whose template has no `O` at all, out of the refusal.
fn cmd_choose_tree_order_is_known(args: &args) -> bool {
    RustSortCriteria::parse_order(args_get_str(args, b'O')) != SORT_END || args_has(args, b'O') == 0
}

unsafe fn cmd_choose_tree_exec(self_0: &cmd, item: &cmdq_item) -> cmd_retval {
    let args = cmd_get_args(self_0);
    if !cmd_choose_tree_order_is_known(args) {
        unsafe { item.error(c"invalid sort order", fmt_args![]) };
        return CMD_RETURN_ERROR;
    }
    if let Some(mode) = cmd_choose_tree_mode(self_0) {
        let target = &item.target;
        let pane = target.pane_ref().expect("the command target has a pane");
        unsafe { pane.set_mode(None, mode, Some(target), Some(args)) };
    }
    CMD_RETURN_NORMAL
}
