//! `find-window`: turns a search into a tree-mode filter and opens the tree on
//! a pane.
//!
//! The command's whole job is translation. `-C`, `-N` and `-T` say whether to
//! look in a pane's content, a window's name and a pane's title — none of them
//! means all three — and `-r` and `-i` say whether the match is a regular
//! expression and whether case matters. What comes out is one format string
//! testing whichever of the three were asked for, joined by `#{||:}`, which is
//! handed to a fresh set of arguments as `-f` and from there to
//! [`WindowMode::Tree`](WindowMode). `-Z` is passed on as a bare
//! flag for the mode's own
//! zoom.
//!
//! Quirks kept:
//!
//! * `-r` does two things at once: it selects the `/r` search modifier *and*
//!   drops the `*`s that would otherwise wrap the string, since a regular
//!   expression anchors itself. `-i` alone selects `/i` and keeps the stars.
//! * The search string is interpolated into the format as it stands, so a
//!   string carrying `,`, `}` or `#{` is read as part of the format rather
//!   than as text to look for.
//! * There is no error branch at all: the routine always answers
//!   [`CMD_RETURN_NORMAL`], and it builds the filter and the arguments even
//!   when [`window_pane_set_mode`] is going to refuse them because the pane is
//!   already in that mode.

use crate::args::RustArguments;
use crate::args::{args_create, args_set, args_string_str};
use crate::cmd::cmd_get_args;
use crate::cmd::cmdq_item;
use crate::cmd::{RustCommandEntry, cmd, cmd_entry_flag, cmd_retval};
use crate::types::{ArgsValue, WindowMode, args_parse_t, args_value_t};
use ::core::ffi::c_char;
use ::std::ffi::CString;

use crate::consts::{CMD_FIND_PANE, CMD_RETURN_NORMAL};

pub(crate) static cmd_find_window_entry: RustCommandEntry = RustCommandEntry {
    name: c"find-window",
    alias: Some(c"findw"),
    args: args_parse_t {
        template: c"CiNrt:TZ",
        lower: 1,
        upper: 1,
        cb: None,
    },
    usage: c"[-CiNrTZ] [-t target-pane] match-string",
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
    exec: cmd_find_window_exec,
};

/// The filter format the tree mode is given, built from the search string and
/// the flags.
///
/// The seven shapes the C spells out as seven `xasprintf` calls are one list
/// and one fold here. Each of `-C`, `-N` and `-T` contributes the format that
/// tests its own place — the pane's content, the window's name, the pane's
/// title — and the list is folded from the right into nested `#{||:a,b}`,
/// which is exactly what those seven strings say: one part on its own, two
/// joined once, three joined as `#{||:C,#{||:N,T}}`.
///
/// The list is never empty, so the fold always has something to start from:
/// a command naming none of the three is read as naming all three.
///
/// The search string is the command's one argument, which the entry's own
/// template makes mandatory — `lower` and `upper` are both one — so the
/// parser has refused the command before exec runs if it is missing, and
/// there is no null to guard here.
unsafe fn cmd_find_window_filter(args: &RustArguments) -> Vec<u8> {
    unsafe {
        let s = args_string_str(args, 0)
            .expect("argument count checked")
            .to_bytes();
        let regex = args.argument_flag_count(b'r') != 0;
        let ignore_case = args.argument_flag_count(b'i') != 0;
        let star: &[u8] = match regex {
            true => b"",
            false => b"*",
        };
        let suffix: &[u8] = match (regex, ignore_case) {
            (true, true) => b"/ri",
            (true, false) => b"/r",
            (false, true) => b"/i",
            (false, false) => b"",
        };

        let mut content = args.argument_flag_count(b'C') != 0;
        let mut name = args.argument_flag_count(b'N') != 0;
        let mut title = args.argument_flag_count(b'T') != 0;
        if !content && !name && !title {
            content = true;
            name = true;
            title = true;
        }

        let mut parts: Vec<Vec<u8>> = Vec::new();
        if content {
            parts.push([b"#{C", suffix, b":", s, b"}"].concat());
        }
        for (wanted, place) in [
            (name, &b"#{window_name}"[..]),
            (title, &b"#{pane_title}"[..]),
        ] {
            if wanted {
                parts.push([b"#{m", suffix, b":", star, s, star, b",", place, b"}"].concat());
            }
        }

        let mut filter = parts.pop().expect("no place to search in");
        while let Some(part) = parts.pop() {
            filter = [b"#{||:", &part[..], b",", &filter[..], b"}"].concat();
        }
        filter
    }
}

unsafe fn cmd_find_window_exec(self_0: &cmd, item: &cmdq_item) -> cmd_retval {
    let args = cmd_get_args(self_0);
    let target = &item.target;
    let pane = target.pane_ref().expect("the command target has a pane");

    let text = unsafe { cmd_find_window_filter(args) };
    let mut filter = Box::new(args_value_t::default());
    filter.value =
        ArgsValue::String(CString::new(text).expect("window filter contains no NUL bytes"));

    let mut new_args = args_create();
    if args.argument_flag_count(b'Z') != 0 {
        unsafe { args_set(&mut new_args, b'Z', None, 0) };
    }
    unsafe { args_set(&mut new_args, b'f', Some(filter), 0) };

    unsafe { pane.set_mode(None, WindowMode::Tree, Some(target), Some(&new_args)) };
    CMD_RETURN_NORMAL
}
